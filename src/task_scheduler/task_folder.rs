use winapi::{shared::{wtypes::BSTR, ntdef::NULL}, um::{taskschd::ITaskDefinition, oaidl::{VARIANT, VARIANTARG}, oleauto::VariantInit, taskschd::{IRegisteredTask, ITaskFolder, TASK_CREATION}}};

use super::{IUnknownWrapper, RegisteredTask, TaskDefinition, TaskLogonType};
use log::debug;

#[inline(always)]
fn empty_variant() -> VARIANT {
    let mut variant: VARIANTARG = unsafe { std::mem::zeroed() };
    unsafe { VariantInit(&mut variant as *mut _ as _) };
    debug!("Initialized empty variant with type {:?}", unsafe {
        variant.n1.n2().vt
    });
    variant
}

pub struct TaskFolder(IUnknownWrapper<ITaskFolder>);

impl From<*mut ITaskFolder> for TaskFolder {
    fn from(folder: *mut ITaskFolder) -> Self {
        Self(folder.into())
    }
}

impl Into<*mut ITaskFolder> for TaskFolder {
    fn into(self) -> *mut ITaskFolder {
        self.0.into()
    }
}

#[allow(dead_code)]
impl TaskFolder {
    pub fn name(&self) -> std::io::Result<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_Name(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn path(&self) -> std::io::Result<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_Path(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn delete_folder<S: AsRef<str>>(&self, subfolder_name: S) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).DeleteFolder(crate::wstr!(subfolder_name.as_ref()), 0))
    }

    pub fn delete_task<S: AsRef<str>>(&self, task_name: S) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).DeleteTask(crate::wstr!(task_name.as_ref()), 0))
    }

    pub fn register_task_definition<S: AsRef<str>>(
        &self,
        task_name: S,
        task_definition: TaskDefinition,
        flags: TASK_CREATION,
        user_id: Option<S>,
        password: Option<S>,
        logon_type: TaskLogonType,
        sddl: Option<S>,
    ) -> std::io::Result<RegisteredTask> {
        let mut registered_task: *mut IRegisteredTask = NULL as _;
        let task_definition: *mut ITaskDefinition = task_definition.into();

        let uid = user_id.map(|uid| {
            let uid_bstr = crate::wstr!(uid.as_ref());
            crate::bstr_variant!(uid_bstr)
        }).unwrap_or_else(empty_variant);

        let password = password.map(|passwd| {
            let pwd_bstr = crate::wstr!(passwd.as_ref());
            crate::bstr_variant!(pwd_bstr)
        }).unwrap_or_else(empty_variant);

        let sddl = sddl.map(|sddl| {
            let sddl_bstr = crate::wstr!(sddl.as_ref());
            crate::bstr_variant!(sddl_bstr)
        }).unwrap_or_else(|| {
            let empty = crate::wstr!("");
            crate::bstr_variant!(empty)
        });

        crate::w32_ok!(self.0.RegisterTaskDefinition(
            crate::wstr!(task_name.as_ref()),
            task_definition as *const _,
            flags as _,
            uid,
            password,
            logon_type as _,
            sddl,
            &mut registered_task
        ))?;

        Ok(registered_task.into())
    }
}
