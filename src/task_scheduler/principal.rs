use winapi::{
    shared::wtypes::BSTR,
    um::taskschd::{
        IPrincipal, TASK_LOGON_GROUP, TASK_LOGON_INTERACTIVE_TOKEN,
        TASK_LOGON_INTERACTIVE_TOKEN_OR_PASSWORD, TASK_LOGON_NONE, TASK_LOGON_PASSWORD,
        TASK_LOGON_S4U, TASK_LOGON_SERVICE_ACCOUNT, TASK_RUNLEVEL_HIGHEST, TASK_RUNLEVEL_LUA,
    },
};

use super::IUnknownWrapper;

#[repr(u32)]
pub enum TaskLogonType {
    None = TASK_LOGON_NONE,
    Password = TASK_LOGON_PASSWORD,
    S4U = TASK_LOGON_S4U,
    InteractiveToken = TASK_LOGON_INTERACTIVE_TOKEN,
    Group = TASK_LOGON_GROUP,
    ServiceAccount = TASK_LOGON_SERVICE_ACCOUNT,
    InteractiveTokenOrPassword = TASK_LOGON_INTERACTIVE_TOKEN_OR_PASSWORD,
}

impl From<u32> for TaskLogonType {
    fn from(value: u32) -> Self {
        match value {
            TASK_LOGON_PASSWORD => Self::Password,
            TASK_LOGON_S4U => Self::S4U,
            TASK_LOGON_INTERACTIVE_TOKEN => Self::InteractiveToken,
            TASK_LOGON_GROUP => Self::Group,
            TASK_LOGON_SERVICE_ACCOUNT => Self::ServiceAccount,
            TASK_LOGON_INTERACTIVE_TOKEN_OR_PASSWORD => Self::InteractiveTokenOrPassword,
            _ => Self::None,
        }
    }
}

#[repr(u32)]
pub enum TaskRunlevel {
    Lua = TASK_RUNLEVEL_LUA,
    Highest = TASK_RUNLEVEL_HIGHEST,
}

impl From<u32> for TaskRunlevel {
    fn from(value: u32) -> Self {
        match value {
            TASK_RUNLEVEL_HIGHEST => Self::Highest,
            _ => Self::Lua,
        }
    }
}

pub struct Principal(IUnknownWrapper<IPrincipal>);

impl From<*mut IPrincipal> for Principal {
    fn from(principal: *mut IPrincipal) -> Self {
        Self(principal.into())
    }
}

impl Into<*mut IPrincipal> for Principal {
    fn into(self) -> *mut IPrincipal {
        self.0.into()
    }
}

#[allow(dead_code)]
impl Principal {
    pub fn id(&self) -> std::io::Result<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_Id(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_id<S: AsRef<str>>(&self, id: S) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).put_Id(crate::wstr!(id.as_ref())))?;
        Ok(())
    }

    pub fn display_name(&self) -> std::io::Result<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_DisplayName(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_display_name<S: AsRef<str>>(&self, display_name: S) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).put_DisplayName(crate::wstr!(display_name.as_ref())))?;
        Ok(())
    }

    pub fn user_id(&self) -> std::io::Result<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_UserId(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_user_id<S: AsRef<str>>(&self, user_id: S) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).put_DisplayName(crate::wstr!(user_id.as_ref())))?;
        Ok(())
    }

    pub fn group_id(&self) -> std::io::Result<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_GroupId(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_group_id<S: AsRef<str>>(&self, group_id: S) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).put_DisplayName(crate::wstr!(group_id.as_ref())))?;
        Ok(())
    }

    pub fn logon_type(&self) -> std::io::Result<TaskLogonType> {
        let mut ret: u32 = 0;
        crate::w32_ok!((*self.0).get_LogonType(&mut ret))?;
        Ok(ret.into())
    }

    pub fn set_logon_type(&self, logon_type: TaskLogonType) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).put_LogonType(logon_type as u32))?;
        Ok(())
    }

    pub fn runlevel(&self) -> std::io::Result<TaskRunlevel> {
        let mut ret: u32 = 0;
        crate::w32_ok!((*self.0).get_RunLevel(&mut ret))?;
        Ok(ret.into())
    }

    pub fn set_runlevel(&self, runlevel: TaskRunlevel) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).put_LogonType(runlevel as u32))?;
        Ok(())
    }
}
