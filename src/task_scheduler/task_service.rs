use winapi::{
    shared::{
        ntdef::NULL,
        rpcdce::{RPC_C_AUTHN_LEVEL_PKT_PRIVACY, RPC_C_IMP_LEVEL_IMPERSONATE},
        wtypes::{BSTR, VARIANT_BOOL, VARIANT_FALSE, VARIANT_TRUE},
        wtypesbase::CLSCTX_INPROC_SERVER,
    },
    um::{
        combaseapi::{CoCreateInstance, CoInitializeEx, CoInitializeSecurity, CoUninitialize},
        oaidl::{VARIANT, VARIANTARG},
        objbase::COINIT_APARTMENTTHREADED,
        oleauto::VariantInit,
        taskschd::{ITaskDefinition, ITaskFolder, ITaskService, TaskScheduler},
    },
};

use log::debug;

use super::{TaskCompatibility, TaskDefinition, TaskFolder};

#[inline(always)]
#[allow(dead_code)]
fn empty_variant() -> VARIANT {
    let mut variant: VARIANTARG = unsafe { std::mem::zeroed() };
    unsafe { VariantInit(&mut variant as *mut _ as _) };
    debug!("Initialized empty variant with type {:?}", unsafe {
        variant.n1.n2().vt
    });
    variant
}

#[derive(Default)]
pub struct WindowsTaskScheduler {
    co_init: bool,
    service: Option<Box<ITaskService>>,
}

#[allow(dead_code)]
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

        let mut task_service: *mut ITaskService = std::ptr::null_mut();
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
            debug!("Service: {:?}", service.lpVtbl);
            crate::w32_ok!(DEBUG service.Connect(
                std::mem::zeroed(),
                std::mem::zeroed(),
                std::mem::zeroed(),
                std::mem::zeroed(),
            ), |result| {
                debug!("ITaskService::Connect() -> {:#X}", result);
            })
        } else {
            Err(std::io::ErrorKind::NotConnected.into())
        }
    }

    pub fn is_connected(&self) -> std::io::Result<bool> {
        if let Some(service) = &self.service {
            let mut ret: VARIANT_BOOL = VARIANT_FALSE;
            crate::w32_ok!(service.get_Connected(&mut ret))?;
            Ok(ret == VARIANT_TRUE)
        } else {
            Ok(false)
        }
    }

    pub fn connected_domain(&self) -> std::io::Result<String> {
        if let Some(service) = &self.service {
            let mut ret: BSTR = std::ptr::null_mut();
            crate::w32_ok!(service.get_ConnectedDomain(&mut ret))?;
            super::bstr_to_string(ret)
        } else {
            Err(std::io::ErrorKind::NotConnected.into())
        }
    }

    pub fn connected_user(&self) -> std::io::Result<String> {
        if let Some(service) = &self.service {
            let mut ret: BSTR = std::ptr::null_mut();
            crate::w32_ok!(service.get_ConnectedUser(&mut ret))?;
            super::bstr_to_string(ret)
        } else {
            Err(std::io::ErrorKind::NotConnected.into())
        }
    }

    pub fn target_server(&self) -> std::io::Result<String> {
        if let Some(service) = &self.service {
            let mut ret: BSTR = std::ptr::null_mut();
            crate::w32_ok!(service.get_TargetServer(&mut ret))?;
            super::bstr_to_string(ret)
        } else {
            Err(std::io::ErrorKind::NotConnected.into())
        }
    }

    pub fn folder<T: AsRef<str>>(&self, folder: T) -> std::io::Result<TaskFolder> {
        if let Some(service) = &self.service {
            let mut task_folder: *mut ITaskFolder = std::ptr::null_mut();
            let task_folder_name = crate::wstr!(folder.as_ref());
            crate::w32_ok!(DEBUG service.GetFolder(
                task_folder_name,
                &mut task_folder as *mut *mut _
            ), |result| debug!(
                "ITaskService::GetFolder({}) -> {:#X}",
                folder.as_ref(), result
            ))?;

            debug!("Task Folder: {:?}", task_folder);

            Ok(task_folder.into())
        } else {
            Err(std::io::ErrorKind::NotConnected.into())
        }
    }

    pub fn new_task(&self) -> std::io::Result<TaskDefinition> {
        if let Some(service) = &self.service {
            let mut task_definition: *mut ITaskDefinition = std::ptr::null_mut();
            crate::w32_ok!(service.NewTask(0, &mut task_definition))?;
            Ok(task_definition.into())
        } else {
            Err(std::io::ErrorKind::NotConnected.into())
        }
    }

    pub fn highest_version(&self) -> std::io::Result<TaskCompatibility> {
        if let Some(service) = &self.service {
            let mut ret: u32 = 0;
            crate::w32_ok!(service.get_HighestVersion(&mut ret))?;
            Ok(ret.into())
        } else {
            Err(std::io::ErrorKind::NotConnected.into())
        }
    }

    // pub fn create_logon_task_as_admin(&mut self, task: super::LogonTaskDefinition, delete_first: bool) -> std::io::Result<()> {
    //     if let Some(service) = &self.service {
    //         if let Some(folder) = &self.folder {
    //             let mut task_name = crate::wstr!(task.task_name);
    //             if delete_first {
    //                 let result = unsafe { folder.DeleteTask(task_name.as_mut_ptr(), 0) };
    //                 debug!("ITaskFolder::DeleteTask() -> {:#X}", result);
    //             }
    //             let mut task_definition: *mut ITaskDefinition = unsafe { std::mem::zeroed() };
    //             crate::w32_ok!(
    //                 DEBUG service.NewTask(0, &mut task_definition as *mut *mut _ as _),
    //                 |result| debug!("ITaskService::NewTask(0, {:?}) -> {:#X}", task_definition, result)
    //             )?;

    //             let mut task_registration_info: *mut IRegistrationInfo = unsafe { std::mem::zeroed() };
    //             crate::w32_ok!(
    //                 DEBUG (*task_definition).get_RegistrationInfo(&mut task_registration_info as *mut *mut _),
    //                 |result| debug!("ITaskDefinition::get_RegistrationInfo({:?) -> {:#X}", task_registration_info, result)
    //             )?;
    //             crate::w32_ok!(
    //                 DEBUG (*task_registration_info).put_Author(crate::wstr!(task.author).as_mut_ptr()),
    //                 |result| debug!("IRegistration::put_Author({}) -> {:#X}", task.author, result)
    //             )?;

    //             if task.start_when_available {
    //                 let mut task_settings: *mut ITaskSettings = unsafe { std::mem::zeroed() };
    //                 crate::w32_ok!(
    //                     DEBUG (*task_definition).get_Settings(&mut task_settings as *mut *mut _),
    //                     |result| debug!("ITaskDefinition::get_Settings({:?}) -> {:#X}", task_settings, result)
    //                 )?;

    //                 crate::w32_ok!(
    //                     DEBUG (*task_settings).put_StartWhenAvailable(VARIANT_TRUE),
    //                     |result| debug!("ITaskSettings::put_StartWhenAvailable(VARIANT_TRUE) -> {:#X}", result)
    //                 )?;
    //                 unsafe {
    //                     (*task_settings).Release();
    //                     std::ptr::drop_in_place(task_settings);
    //                 };
    //             }

    //             let mut trigger_collection: *mut ITriggerCollection = unsafe { std::mem::zeroed() };
    //             crate::w32_ok!(
    //                 DEBUG (*task_definition).get_Triggers(&mut trigger_collection as *mut *mut _),
    //                 |result| debug!("ITaskDefinition::get_Triggers({:?}) -> {:#X}", trigger_collection, result)
    //             )?;
    //             let mut trigger: *mut ITrigger = unsafe { std::mem::zeroed() };
    //             crate::w32_ok!(
    //                 DEBUG (*trigger_collection).Create(TASK_TRIGGER_LOGON, &mut trigger as *mut *mut _),
    //                 |result| debug!("ITriggerCollection::Create(TASK_TRIGGER_LOGON, {:?}) -> {:#X}", trigger, result)
    //             )?;
    //             unsafe {
    //                 (*trigger_collection).Release();
    //                 std::ptr::drop_in_place(trigger_collection);
    //             }

    //             use winapi::Interface as _;
    //             let mut logon_trigger: *mut ILogonTrigger = unsafe { std::mem::zeroed() };
    //             crate::w32_ok!(
    //                 DEBUG (*trigger).QueryInterface(&ILogonTrigger::uuidof() as _, &mut logon_trigger as *mut *mut _ as _),
    //                 |result| debug!("ITrigger::QueryInterface(ILogonTrigger::uuidof(), {:?}) -> {:#X}", logon_trigger, result)
    //             )?;

    //             unsafe {
    //                 (*trigger).Release();
    //                 std::ptr::drop_in_place(trigger);
    //             }

    //             crate::w32_ok!((*logon_trigger).put_Id(crate::wstr!(task.task_name).as_mut_ptr()))?;
    //             crate::w32_ok!((*logon_trigger).put_UserId(crate::wstr!("Administrator").as_mut_ptr()))?;
    //             unsafe {
    //                 (*logon_trigger).Release();
    //                 std::ptr::drop_in_place(logon_trigger);
    //             }

    //             Ok(())
    //         } else {
    //             Err(std::io::ErrorKind::NotConnected.into())
    //         }
    //     } else {
    //         Err(std::io::ErrorKind::NotConnected.into())
    //     }
    // }
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

        if self.co_init {
            unsafe {
                CoUninitialize();
            }
        }
    }
}
