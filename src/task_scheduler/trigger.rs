use winapi::{
    shared::{
        guiddef::GUID,
        wtypes::{BSTR, VARIANT_BOOL, VARIANT_FALSE, VARIANT_TRUE},
    },
    um::taskschd::{
        ILogonTrigger, ITrigger, ITriggerCollection, TASK_TRIGGER_BOOT, TASK_TRIGGER_DAILY,
        TASK_TRIGGER_EVENT, TASK_TRIGGER_IDLE, TASK_TRIGGER_LOGON, TASK_TRIGGER_MONTHLY,
        TASK_TRIGGER_MONTHLYDOW, TASK_TRIGGER_REGISTRATION, TASK_TRIGGER_SESSION_STATE_CHANGE,
        TASK_TRIGGER_TIME, TASK_TRIGGER_TYPE2, TASK_TRIGGER_WEEKLY,
    },
};

use super::IUnknownWrapper;

#[repr(u32)]
pub enum TaskTriggerType {
    Event = TASK_TRIGGER_EVENT,
    Time = TASK_TRIGGER_TIME,
    Daily = TASK_TRIGGER_DAILY,
    Weekly = TASK_TRIGGER_WEEKLY,
    Monthly = TASK_TRIGGER_MONTHLY,
    MonthlyDow = TASK_TRIGGER_MONTHLYDOW,
    Idle = TASK_TRIGGER_IDLE,
    Registration = TASK_TRIGGER_REGISTRATION,
    Boot = TASK_TRIGGER_BOOT,
    Logon = TASK_TRIGGER_LOGON,
    SessionStateChange = TASK_TRIGGER_SESSION_STATE_CHANGE,
}

impl From<TASK_TRIGGER_TYPE2> for TaskTriggerType {
    fn from(t: TASK_TRIGGER_TYPE2) -> Self {
        match t {
            TASK_TRIGGER_EVENT => Self::Event,
            TASK_TRIGGER_TIME => Self::Time,
            TASK_TRIGGER_DAILY => Self::Daily,
            TASK_TRIGGER_WEEKLY => Self::Weekly,
            TASK_TRIGGER_MONTHLY => Self::Monthly,
            TASK_TRIGGER_MONTHLYDOW => Self::MonthlyDow,
            TASK_TRIGGER_IDLE => Self::Idle,
            TASK_TRIGGER_REGISTRATION => Self::Registration,
            TASK_TRIGGER_BOOT => Self::Boot,
            TASK_TRIGGER_LOGON => Self::Logon,
            TASK_TRIGGER_SESSION_STATE_CHANGE => Self::SessionStateChange,
            _ => unreachable!("Uh-oh, this shouldn't happen"),
        }
    }
}

pub struct TriggerCollection(IUnknownWrapper<ITriggerCollection>);

impl From<*mut ITriggerCollection> for TriggerCollection {
    fn from(trigger_collection: *mut ITriggerCollection) -> Self {
        Self(trigger_collection.into())
    }
}

impl Into<*mut ITriggerCollection> for TriggerCollection {
    fn into(self) -> *mut ITriggerCollection {
        self.0.into()
    }
}

#[allow(dead_code)]
impl TriggerCollection {
    pub fn clear(&self) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).Clear())?;
        Ok(())
    }

    pub fn count(&self) -> std::io::Result<usize> {
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

    pub fn remove(&self, index: usize) -> std::io::Result<()> {
        let count = self.count()?;
        if index == 0 || index > count {
            return Err(std::io::ErrorKind::InvalidInput.into());
        }

        crate::w32_ok!((*self.0).Remove(Self::index_to_variant(index)))?;

        Ok(())
    }

    pub fn get(&self, index: usize) -> std::io::Result<Trigger> {
        let count = self.count()?;
        if index == 0 || index > count {
            return Err(std::io::ErrorKind::InvalidInput.into());
        }

        let mut trigger: *mut ITrigger = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_Item(index as i32, &mut trigger))?;
        Ok(trigger.into())
    }

    pub fn create(&self, trigger_type: TaskTriggerType) -> std::io::Result<Trigger> {
        let mut trigger: *mut ITrigger = std::ptr::null_mut();
        crate::w32_ok!((*self.0).Create(trigger_type as _, &mut trigger))?;
        Ok(trigger.into())
    }
}

pub struct Trigger(IUnknownWrapper<ITrigger>);

impl From<*mut ITrigger> for Trigger {
    fn from(trigger: *mut ITrigger) -> Self {
        Self(trigger.into())
    }
}

impl Into<*mut ITrigger> for Trigger {
    fn into(self) -> *mut ITrigger {
        self.0.into()
    }
}

impl std::ops::Deref for Trigger {
    type Target = ITrigger;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Trigger {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[allow(dead_code)]
impl Trigger {
    pub fn id(&self) -> std::io::Result<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_Id(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_id<S: AsRef<str>>(&self, id: S) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).put_Id(crate::wstr!(id.as_ref())))?;
        Ok(())
    }

    pub fn enabled(&self) -> std::io::Result<bool> {
        let mut ret: VARIANT_BOOL = VARIANT_FALSE;
        crate::w32_ok!((*self.0).get_Enabled(&mut ret))?;
        Ok(ret == VARIANT_TRUE)
    }

    pub fn set_enabled(&self, allow: bool) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).put_Enabled(if allow { VARIANT_TRUE } else { VARIANT_FALSE }))?;
        Ok(())
    }

    pub fn execution_time_limit(&self) -> std::io::Result<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_ExecutionTimeLimit(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_execution_time_limit<S: AsRef<str>>(&self, time_limit: S) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).put_ExecutionTimeLimit(crate::wstr!(time_limit.as_ref())))?;
        Ok(())
    }

    pub fn start_boundary(&self) -> std::io::Result<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_StartBoundary(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_start_boundary<S: AsRef<str>>(&self, boundary: S) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).put_StartBoundary(crate::wstr!(boundary.as_ref())))?;
        Ok(())
    }

    pub fn end_boundary(&self) -> std::io::Result<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_EndBoundary(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_end_boundary<S: AsRef<str>>(&self, boundary: S) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).put_EndBoundary(crate::wstr!(boundary.as_ref())))?;
        Ok(())
    }
}

crate::generate_trigger_type!(TaskTriggerType::Logon, ILogonTrigger, LogonTrigger);

pub trait SubTrigger {
    fn new(trigger: Trigger) -> std::io::Result<Self>
    where
        Self: Sized;
    fn uuid() -> GUID;
    fn get_type() -> TaskTriggerType;
    fn trigger(&self) -> &Trigger;
}

#[macro_export(local_inner_macros)]
macro_rules! generate_trigger_type {
    ($enum:expr, $target:ident, $newt:ident) => {
        pub struct $newt {
            target: Box<$target>,
            trigger: Trigger,
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

        impl SubTrigger for $newt {
            fn new(trigger: Trigger) -> std::io::Result<Self> {
                let mut ret_trigger: *mut $target = std::ptr::null_mut();
                use winapi::Interface as _;
                crate::w32_ok!(trigger.QueryInterface(
                    &$target::uuidof() as _,
                    &mut ret_trigger as *mut *mut _ as _
                ))?;
                Ok(Self {
                    target: unsafe { Box::from_raw(ret_trigger) },
                    trigger,
                })
            }

            fn trigger(&self) -> &Trigger {
                &self.trigger
            }

            fn uuid() -> GUID {
                use winapi::Interface as _;
                $target::uuidof()
            }

            fn get_type() -> TaskTriggerType {
                $enum
            }
        }
    };
}

#[allow(dead_code)]
impl LogonTrigger {
    pub fn delay(&self) -> std::io::Result<String> {
        let mut delay_bstr: BSTR = std::ptr::null_mut();
        crate::w32_ok!(self.get_Delay(&mut delay_bstr))?;
        super::bstr_to_string(delay_bstr)
    }

    pub fn set_delay<S: AsRef<str>>(&self, delay: S) -> std::io::Result<()> {
        crate::w32_ok!(self.put_Delay(crate::wstr!(delay.as_ref())))?;
        Ok(())
    }

    pub fn user_id(&self) -> std::io::Result<String> {
        let mut user_id_bstr: BSTR = std::ptr::null_mut();
        crate::w32_ok!(self.get_UserId(&mut user_id_bstr))?;
        super::bstr_to_string(user_id_bstr)
    }

    pub fn set_user_id<S: AsRef<str>>(&self, user_id: S) -> std::io::Result<()> {
        crate::w32_ok!(self.put_UserId(crate::wstr!(user_id.as_ref())))?;
        Ok(())
    }
}
