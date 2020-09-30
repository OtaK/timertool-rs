use winapi::{
    um::{
        objbase::COINIT_APARTMENTTHREADED,
        combaseapi::{
            CoInitializeEx, CoInitializeSecurity, CoUninitialize, CoCreateInstance,
        },
        taskschd::{
            ITaskFolder, ITaskService, TaskScheduler,
            ITaskDefinition, IRegistrationInfo, ITaskSettings,
            ITriggerCollection, ITrigger, ILogonTrigger,
            IActionCollection, IAction, IExecAction,
            IRegisteredTask, TASK_TRIGGER_LOGON, TASK_ACTION_EXEC,
            TASK_CREATE_OR_UPDATE, TASK_LOGON_GROUP,
        },
        oaidl::{VARIANT, VARIANTARG},
        oleauto::VariantInit,
    },
    shared::{
        wtypes::VARIANT_TRUE,
        wtypesbase::CLSCTX_INPROC_SERVER,
        rpcdce::{
            RPC_C_AUTHN_LEVEL_PKT_PRIVACY,
            RPC_C_IMP_LEVEL_IMPERSONATE
        },
        ntdef::NULL,
    },
};

use log::{debug, info};

#[inline(always)]
fn empty_variant() -> VARIANT {
    let mut variant: VARIANTARG = unsafe { std::mem::zeroed() };
    unsafe { VariantInit(&mut variant as *mut _ as _) };
    debug!("Initialized empty variant with type {:?}", unsafe { variant.n1.n2().vt });
    variant
}

#[derive(Default)]
pub struct WindowsTaskScheduler {
    co_init: bool,
    service: Option<Box<ITaskService>>,
    folder: Option<Box<ITaskFolder>>,
    task: Option<Box<ITaskDefinition>>,
}

impl WindowsTaskScheduler {
    pub fn new() -> std::io::Result<Self> {
        let mut ret = Self::default();
        crate::w32_ok!(DEBUG CoInitializeEx(
            NULL,
            COINIT_APARTMENTTHREADED
        ), |result| debug!(
            "CoInitializeEx(NULL, COINIT_APARTMENTTHREADED) -> {}", result
        ))?;

        ret.co_init = true;

        crate::w32_ok!(DEBUG CoInitializeSecurity(
            NULL,
            -1,
            NULL as _,
            NULL,
            RPC_C_AUTHN_LEVEL_PKT_PRIVACY,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            NULL,
            0,
            NULL,
        ), |result| debug!(
            r#"CoInitializeSecurity(
                NULL,
                -1,
                NULL,
                NULL,
                RPC_C_AUTHN_LEVEL_PKT_PRIVACY,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                NULL,
                0,
                NULL
            ) -> {}"#, result
        ))?;

        use winapi::{Class as _, Interface as _};

        let mut task_service: *mut ITaskService = unsafe { std::mem::zeroed() };
        crate::w32_ok!(DEBUG CoCreateInstance(
            &TaskScheduler::uuidof(),
            NULL as _,
            CLSCTX_INPROC_SERVER,
            &ITaskService::uuidof(),
            &mut task_service as *mut *mut ITaskService as _,
        ), |result| debug!(
            r#"CoCreateInstance(
                TaskScheduler::uuidof(),
                NULL,
                CLSCTX_INPROC_SERVER,
                ITaskService::uuidof(),
                &task_service
            ) -> {:#X}"#,
            result
        ))?;

        ret.service = Some(unsafe { Box::from_raw(task_service) });
        Ok(ret)
    }

    pub fn connect(&self) -> std::io::Result<()> {
        if let Some(service) = &self.service {
            let e = empty_variant();
            debug!("Service: {:?}", service.lpVtbl);
            crate::w32_ok!(DEBUG service.Connect(
                e,
                e,
                e,
                e,
            ), |result| {
                debug!("ITaskService::Connect() -> {:#X}", result);
            })
        } else {
            Err(std::io::ErrorKind::NotConnected.into())
        }
    }

    pub fn fetch_folder<T: AsRef<str>>(&mut self, folder: T) -> std::io::Result<()> {
        if let Some(service) = &self.service {
            let mut task_folder: *mut ITaskFolder = unsafe { std::mem::zeroed() };
            let mut task_folder_name = crate::wstr!(folder.as_ref());
            crate::w32_ok!(DEBUG service.GetFolder(
                task_folder_name.as_mut_ptr(),
                &mut task_folder as *mut *mut _
            ), |result| debug!(
                "ITaskService::GetFolder({}) -> {:#X}",
                folder.as_ref(), result
            ))?;

            debug!("Task Folder: {:?}", task_folder);

            self.folder = Some(unsafe { Box::from_raw(task_folder) });

            Ok(())
        } else {
            Err(std::io::ErrorKind::NotConnected.into())
        }
    }

    pub fn create_logon_task_as_admin(&mut self, task: super::LogonTaskDefinition, delete_first: bool) -> std::io::Result<()> {
        if let Some(service) = &self.service {
            if let Some(folder) = &self.folder {
                let mut task_name = crate::wstr!(task.task_name);
                if delete_first {
                    let result = unsafe { folder.DeleteTask(task_name.as_mut_ptr(), 0) };
                    debug!("ITaskFolder::DeleteTask() -> {:#X}", result);
                }
                let mut task_definition: *mut ITaskDefinition = unsafe { std::mem::zeroed() };
                crate::w32_ok!(
                    DEBUG service.NewTask(0, &mut task_definition as *mut *mut _ as _),
                    |result| debug!("ITaskService::NewTask(0, {:?}) -> {:#X}", task_definition, result)
                )?;

                let mut task_registration_info: *mut IRegistrationInfo = unsafe { std::mem::zeroed() };
                crate::w32_ok!(
                    DEBUG (*task_definition).get_RegistrationInfo(&mut task_registration_info as *mut *mut _),
                    |result| debug!("ITaskDefinition::get_RegistrationInfo({:?) -> {:#X}", task_registration_info, result)
                )?;
                crate::w32_ok!(
                    DEBUG (*task_registration_info).put_Author(crate::wstr!(task.author).as_mut_ptr()),
                    |result| debug!("IRegistration::put_Author({}) -> {:#X}", task.author, result)
                )?;

                if task.start_when_available {
                    let mut task_settings: *mut ITaskSettings = unsafe { std::mem::zeroed() };
                    crate::w32_ok!(
                        DEBUG (*task_definition).get_Settings(&mut task_settings as *mut *mut _),
                        |result| debug!("ITaskDefinition::get_Settings({:?}) -> {:#X}", task_settings, result)
                    )?;

                    crate::w32_ok!(
                        DEBUG (*task_settings).put_StartWhenAvailable(VARIANT_TRUE),
                        |result| debug!("ITaskSettings::put_StartWhenAvailable(VARIANT_TRUE) -> {:#X}", result)
                    )?;
                    unsafe {
                        (*task_settings).Release();
                        std::ptr::drop_in_place(task_settings);
                    };
                }

                let mut trigger_collection: *mut ITriggerCollection = unsafe { std::mem::zeroed() };
                crate::w32_ok!(
                    DEBUG (*task_definition).get_Triggers(&mut trigger_collection as *mut *mut _),
                    |result| debug!("ITaskDefinition::get_Triggers({:?}) -> {:#X}", trigger_collection, result)
                )?;
                let mut trigger: *mut ITrigger = unsafe { std::mem::zeroed() };
                crate::w32_ok!(
                    DEBUG (*trigger_collection).Create(TASK_TRIGGER_LOGON, &mut trigger as *mut *mut _),
                    |result| debug!("ITriggerCollection::Create(TASK_TRIGGER_LOGON, {:?}) -> {:#X}", trigger, result)
                )?;
                unsafe {
                    (*trigger_collection).Release();
                    std::ptr::drop_in_place(trigger_collection);
                }

                use winapi::Interface as _;
                let mut logon_trigger: *mut ILogonTrigger = unsafe { std::mem::zeroed() };
                crate::w32_ok!(
                    DEBUG (*trigger).QueryInterface(&ILogonTrigger::uuidof() as _, &mut logon_trigger as *mut *mut _ as _),
                    |result| debug!("ITrigger::QueryInterface(ILogonTrigger::uuidof(), {:?}) -> {:#X}", logon_trigger, result)
                )?;

                unsafe {
                    (*trigger).Release();
                    std::ptr::drop_in_place(trigger);
                }

                crate::w32_ok!((*logon_trigger).put_Id(crate::wstr!(task.task_name).as_mut_ptr()))?;
                crate::w32_ok!((*logon_trigger).put_UserId(crate::wstr!("Administrator").as_mut_ptr()))?;
                unsafe {
                    (*logon_trigger).Release();
                    std::ptr::drop_in_place(logon_trigger);
                }


                Ok(())
            } else {
                Err(std::io::ErrorKind::NotConnected.into())
            }
        } else {
            Err(std::io::ErrorKind::NotConnected.into())
        }
    }
}

impl Drop for WindowsTaskScheduler {
    fn drop(&mut self) {
        if let Some(service) = self.service.take() {
            unsafe {
                let ptr = Box::into_raw(service);
                (*ptr).Release();
                std::ptr::drop_in_place(ptr);
            }
        }

        if let Some(folder) = self.folder.take() {
            unsafe {
                let ptr = Box::into_raw(folder);
                (*ptr).Release();
                std::ptr::drop_in_place(ptr);
            }
        }

        if let Some(task) = self.task.take() {
            unsafe {
                let ptr = Box::into_raw(task);
                (*ptr).Release();
                std::ptr::drop_in_place(ptr);
            }
        }

        if self.co_init {
            unsafe { CoUninitialize(); }
        }
    }
}
