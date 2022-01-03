use winapi::{
    shared::{guiddef::GUID, wtypes::BSTR},
    um::taskschd::{
        IAction, IActionCollection, IExecAction, TASK_ACTION_COM_HANDLER, TASK_ACTION_EXEC,
        TASK_ACTION_SEND_EMAIL, TASK_ACTION_SHOW_MESSAGE, TASK_ACTION_TYPE,
    },
};

use super::IUnknownWrapper;

#[repr(u32)]
pub enum TaskActionType {
    Exec = TASK_ACTION_EXEC,
    ComHandler = TASK_ACTION_COM_HANDLER,
    SendEmail = TASK_ACTION_SEND_EMAIL,
    ShowMessage = TASK_ACTION_SHOW_MESSAGE,
}

impl From<TASK_ACTION_TYPE> for TaskActionType {
    fn from(t: TASK_ACTION_TYPE) -> Self {
        match t {
            TASK_ACTION_EXEC => Self::Exec,
            TASK_ACTION_COM_HANDLER => Self::ComHandler,
            TASK_ACTION_SEND_EMAIL => Self::SendEmail,
            TASK_ACTION_SHOW_MESSAGE => Self::ShowMessage,
            _ => unreachable!("Uh-oh, this shouldn't happen"),
        }
    }
}

pub struct ActionCollection(IUnknownWrapper<IActionCollection>);

impl From<*mut IActionCollection> for ActionCollection {
    fn from(actions: *mut IActionCollection) -> Self {
        Self(actions.into())
    }
}

impl Into<*mut IActionCollection> for ActionCollection {
    fn into(self) -> *mut IActionCollection {
        self.0.into()
    }
}

#[allow(dead_code)]
impl ActionCollection {
    pub fn clear(&self) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!((*self.0).Clear())?;
        Ok(())
    }

    pub fn count(&self) -> crate::task_scheduler::TaskSchedulerResult<usize> {
        let mut count = 0;
        crate::w32_ok!((*self.0).get_Count(&mut count))?;
        Ok(count as usize)
    }

    #[inline(always)]
    fn index_to_variant(index: usize) -> winapi::um::oaidl::VARIANTARG {
        let mut variant: winapi::um::oaidl::VARIANTARG = unsafe { std::mem::zeroed() };
        unsafe {
            winapi::um::oleauto::VariantInit(&mut variant as *mut _ as _);
            let mut v2 = variant.n1.n2_mut();
            v2.vt = winapi::shared::wtypes::VT_INT as u16;
            (*v2.n3.lVal_mut()) = index as i32;
        };

        variant
    }

    pub fn remove(&self, index: usize) -> crate::task_scheduler::TaskSchedulerResult<()> {
        let count = self.count()?;
        if index == 0 || index > count {
            return Err(std::io::ErrorKind::InvalidInput.into());
        }

        crate::w32_ok!((*self.0).Remove(Self::index_to_variant(index)))?;

        Ok(())
    }

    pub fn get(&self, index: usize) -> crate::task_scheduler::TaskSchedulerResult<Action> {
        let count = self.count()?;
        if index == 0 || index > count {
            return Err(std::io::ErrorKind::InvalidInput.into());
        }

        let mut action: *mut IAction = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_Item(index as i32, &mut action))?;
        Ok(action.into())
    }

    pub fn create(
        &self,
        trigger_type: TaskActionType,
    ) -> crate::task_scheduler::TaskSchedulerResult<Action> {
        let mut trigger: *mut IAction = std::ptr::null_mut();
        crate::w32_ok!((*self.0).Create(trigger_type as _, &mut trigger))?;
        Ok(trigger.into())
    }
}

pub struct Action(IUnknownWrapper<IAction>);

impl From<*mut IAction> for Action {
    fn from(action: *mut IAction) -> Self {
        Self(action.into())
    }
}

impl Into<*mut IAction> for Action {
    fn into(self) -> *mut IAction {
        self.0.into()
    }
}

impl std::ops::Deref for Action {
    type Target = IAction;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Action {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[allow(dead_code)]
impl Action {
    pub fn id(&self) -> crate::task_scheduler::TaskSchedulerResult<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!(self.get_Id(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_id<S: AsRef<str>>(&self, id: S) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!(self.put_Id(crate::wstr!(id.as_ref())))?;
        Ok(())
    }

    pub fn get_type(&self) -> crate::task_scheduler::TaskSchedulerResult<TaskActionType> {
        let mut action_type = 0u32;
        crate::w32_ok!(self.get_Type(&mut action_type))?;
        Ok(action_type.into())
    }
}

crate::generate_action_type!(TaskActionType::Exec, IExecAction, ExecAction);

pub trait SubAction {
    fn new(action: Action) -> crate::task_scheduler::TaskSchedulerResult<Self>
    where
        Self: Sized;
    fn uuid() -> GUID;
    fn get_type() -> TaskActionType;
    fn action(&self) -> &Action;
}

#[macro_export(local_inner_macros)]
macro_rules! generate_action_type {
    ($enum:expr, $target:ident, $newt:ident) => {
        pub struct $newt {
            target: Box<$target>,
            action: Action,
        }

        impl std::ops::Deref for $newt {
            type Target = $target;
            fn deref(&self) -> &Self::Target {
                &self.target
            }
        }

        impl std::ops::DerefMut for $newt {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.target
            }
        }

        impl SubAction for $newt {
            fn new(action: Action) -> crate::task_scheduler::TaskSchedulerResult<Self> {
                let mut ret_action: *mut $target = std::ptr::null_mut();
                use winapi::Interface as _;
                crate::w32_ok!(action
                    .QueryInterface(&$target::uuidof() as _, &mut ret_action as *mut *mut _ as _))?;
                Ok(Self {
                    target: unsafe { Box::from_raw(ret_action) },
                    action,
                })
            }

            fn action(&self) -> &Action {
                &self.action
            }

            fn uuid() -> GUID {
                use winapi::Interface as _;
                $target::uuidof()
            }

            fn get_type() -> TaskActionType {
                $enum
            }
        }
    };
}

#[allow(dead_code)]
impl ExecAction {
    pub fn path(&self) -> crate::task_scheduler::TaskSchedulerResult<String> {
        let mut delay_bstr: BSTR = std::ptr::null_mut();
        crate::w32_ok!(self.get_Path(&mut delay_bstr))?;
        super::bstr_to_string(delay_bstr)
    }

    pub fn set_path<S: AsRef<str>>(
        &self,
        path: S,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!(self.put_Path(crate::wstr!(path.as_ref())))?;
        Ok(())
    }

    pub fn arguments(&self) -> crate::task_scheduler::TaskSchedulerResult<String> {
        let mut delay_bstr: BSTR = std::ptr::null_mut();
        crate::w32_ok!(self.get_Arguments(&mut delay_bstr))?;
        super::bstr_to_string(delay_bstr)
    }

    pub fn set_arguments<S: AsRef<str>>(
        &self,
        arguments: S,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!(self.put_Arguments(crate::wstr!(arguments.as_ref())))?;
        Ok(())
    }

    pub fn working_directory(&self) -> crate::task_scheduler::TaskSchedulerResult<String> {
        let mut delay_bstr: BSTR = std::ptr::null_mut();
        crate::w32_ok!(self.get_WorkingDirectory(&mut delay_bstr))?;
        super::bstr_to_string(delay_bstr)
    }

    pub fn set_working_directory<S: AsRef<str>>(
        &self,
        working_directory: S,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!(self.put_WorkingDirectory(crate::wstr!(working_directory.as_ref())))?;
        Ok(())
    }
}
