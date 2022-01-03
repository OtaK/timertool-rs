use winapi::{
    shared::wtypes::{BSTR, VARIANT_BOOL, VARIANT_FALSE, VARIANT_TRUE},
    um::taskschd::{
        ITaskSettings, TASK_COMPATIBILITY_AT, TASK_COMPATIBILITY_V1, TASK_COMPATIBILITY_V2,
        TASK_COMPATIBILITY_V2_1, TASK_COMPATIBILITY_V2_2, TASK_COMPATIBILITY_V2_3,
        TASK_COMPATIBILITY_V2_4, TASK_INSTANCES_IGNORE_NEW, TASK_INSTANCES_PARALLEL,
        TASK_INSTANCES_QUEUE, TASK_INSTANCES_STOP_EXISTING,
    },
};

#[repr(u32)]
pub enum TaskInstancesPolicy {
    Parallel = TASK_INSTANCES_PARALLEL,
    Queue = TASK_INSTANCES_QUEUE,
    IgnoreNew = TASK_INSTANCES_IGNORE_NEW,
    StopExisting = TASK_INSTANCES_STOP_EXISTING,
}

impl From<u32> for TaskInstancesPolicy {
    fn from(value: u32) -> Self {
        match value {
            TASK_INSTANCES_QUEUE => Self::Queue,
            TASK_INSTANCES_IGNORE_NEW => Self::IgnoreNew,
            TASK_INSTANCES_STOP_EXISTING => Self::StopExisting,
            _ => Self::Parallel,
        }
    }
}

#[repr(u32)]
pub enum TaskCompatibility {
    AT = TASK_COMPATIBILITY_AT,
    V1 = TASK_COMPATIBILITY_V1,
    V2 = TASK_COMPATIBILITY_V2,
    V21 = TASK_COMPATIBILITY_V2_1,
    V22 = TASK_COMPATIBILITY_V2_2,
    V23 = TASK_COMPATIBILITY_V2_3,
    V24 = TASK_COMPATIBILITY_V2_4,
}

impl From<u32> for TaskCompatibility {
    fn from(value: u32) -> Self {
        match value {
            TASK_COMPATIBILITY_AT => Self::AT,
            TASK_COMPATIBILITY_V1 => Self::V1,
            TASK_COMPATIBILITY_V2_1 => Self::V21,
            TASK_COMPATIBILITY_V2_2 => Self::V22,
            TASK_COMPATIBILITY_V2_3 => Self::V23,
            TASK_COMPATIBILITY_V2_4 => Self::V24,
            _ => Self::V2,
        }
    }
}

pub struct TaskSettings(Box<ITaskSettings>);

impl From<*mut ITaskSettings> for TaskSettings {
    fn from(folder: *mut ITaskSettings) -> Self {
        Self(unsafe { Box::from_raw(folder) })
    }
}

impl Into<*mut ITaskSettings> for TaskSettings {
    fn into(self) -> *mut ITaskSettings {
        Box::into_raw(self.0)
    }
}

#[allow(dead_code)]
impl TaskSettings {
    pub fn allow_demand_start(&self) -> crate::task_scheduler::TaskSchedulerResult<bool> {
        let mut ret: VARIANT_BOOL = VARIANT_FALSE;
        crate::w32_ok!((*self.0).get_AllowDemandStart(&mut ret))?;
        Ok(ret == VARIANT_TRUE)
    }

    pub fn set_allow_demand_start(
        &self,
        allow: bool,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!((*self.0).put_AllowDemandStart(if allow {
            VARIANT_TRUE
        } else {
            VARIANT_FALSE
        }))?;
        Ok(())
    }

    pub fn start_when_available(&self) -> crate::task_scheduler::TaskSchedulerResult<bool> {
        let mut ret: VARIANT_BOOL = VARIANT_FALSE;
        crate::w32_ok!((*self.0).get_StartWhenAvailable(&mut ret))?;
        Ok(ret == VARIANT_TRUE)
    }

    pub fn set_start_when_available(
        &self,
        allow: bool,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!((*self.0).put_StartWhenAvailable(if allow {
            VARIANT_TRUE
        } else {
            VARIANT_FALSE
        }))?;
        Ok(())
    }

    pub fn stop_if_going_into_batteries(&self) -> crate::task_scheduler::TaskSchedulerResult<bool> {
        let mut ret: VARIANT_BOOL = VARIANT_FALSE;
        crate::w32_ok!((*self.0).get_StopIfGoingOnBatteries(&mut ret))?;
        Ok(ret == VARIANT_TRUE)
    }

    pub fn set_stop_if_going_into_batteries(
        &self,
        allow: bool,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!((*self.0).put_StopIfGoingOnBatteries(if allow {
            VARIANT_TRUE
        } else {
            VARIANT_FALSE
        }))?;
        Ok(())
    }

    pub fn disallow_start_if_on_batteries(
        &self,
    ) -> crate::task_scheduler::TaskSchedulerResult<bool> {
        let mut ret: VARIANT_BOOL = VARIANT_FALSE;
        crate::w32_ok!((*self.0).get_DisallowStartIfOnBatteries(&mut ret))?;
        Ok(ret == VARIANT_TRUE)
    }

    pub fn set_disallow_start_if_on_batteries(
        &self,
        allow: bool,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!((*self.0).put_DisallowStartIfOnBatteries(if allow {
            VARIANT_TRUE
        } else {
            VARIANT_FALSE
        }))?;
        Ok(())
    }

    pub fn allow_hard_terminate(&self) -> crate::task_scheduler::TaskSchedulerResult<bool> {
        let mut ret: VARIANT_BOOL = VARIANT_FALSE;
        crate::w32_ok!((*self.0).get_AllowHardTerminate(&mut ret))?;
        Ok(ret == VARIANT_TRUE)
    }

    pub fn set_allow_hard_terminate(
        &self,
        allow: bool,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!((*self.0).put_AllowHardTerminate(if allow {
            VARIANT_TRUE
        } else {
            VARIANT_FALSE
        }))?;
        Ok(())
    }

    pub fn run_only_if_network_available(
        &self,
    ) -> crate::task_scheduler::TaskSchedulerResult<bool> {
        let mut ret: VARIANT_BOOL = VARIANT_FALSE;
        crate::w32_ok!((*self.0).get_RunOnlyIfNetworkAvailable(&mut ret))?;
        Ok(ret == VARIANT_TRUE)
    }

    pub fn set_run_only_if_network_available(
        &self,
        allow: bool,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!((*self.0).put_RunOnlyIfNetworkAvailable(if allow {
            VARIANT_TRUE
        } else {
            VARIANT_FALSE
        }))?;
        Ok(())
    }

    pub fn enabled(&self) -> crate::task_scheduler::TaskSchedulerResult<bool> {
        let mut ret: VARIANT_BOOL = VARIANT_FALSE;
        crate::w32_ok!((*self.0).get_Enabled(&mut ret))?;
        Ok(ret == VARIANT_TRUE)
    }

    pub fn set_enabled(&self, allow: bool) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!((*self.0).put_Enabled(if allow { VARIANT_TRUE } else { VARIANT_FALSE }))?;
        Ok(())
    }

    pub fn hidden(&self) -> crate::task_scheduler::TaskSchedulerResult<bool> {
        let mut ret: VARIANT_BOOL = VARIANT_FALSE;
        crate::w32_ok!((*self.0).get_Hidden(&mut ret))?;
        Ok(ret == VARIANT_TRUE)
    }

    pub fn set_hidden(&self, allow: bool) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!((*self.0).put_Hidden(if allow { VARIANT_TRUE } else { VARIANT_FALSE }))?;
        Ok(())
    }

    pub fn run_only_if_idle(&self) -> crate::task_scheduler::TaskSchedulerResult<bool> {
        let mut ret: VARIANT_BOOL = VARIANT_FALSE;
        crate::w32_ok!((*self.0).get_RunOnlyIfIdle(&mut ret))?;
        Ok(ret == VARIANT_TRUE)
    }

    pub fn set_run_only_if_idle(
        &self,
        allow: bool,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!((*self.0).put_RunOnlyIfIdle(if allow {
            VARIANT_TRUE
        } else {
            VARIANT_FALSE
        }))?;
        Ok(())
    }

    pub fn wake_to_run(&self) -> crate::task_scheduler::TaskSchedulerResult<bool> {
        let mut ret: VARIANT_BOOL = VARIANT_FALSE;
        crate::w32_ok!((*self.0).get_WakeToRun(&mut ret))?;
        Ok(ret == VARIANT_TRUE)
    }

    pub fn set_wake_to_run(&self, allow: bool) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!((*self.0).put_WakeToRun(if allow { VARIANT_TRUE } else { VARIANT_FALSE }))?;
        Ok(())
    }

    pub fn execution_time_limit(&self) -> crate::task_scheduler::TaskSchedulerResult<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_ExecutionTimeLimit(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_execution_time_limit<S: AsRef<str>>(
        &self,
        time_limit: S,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!((*self.0).put_ExecutionTimeLimit(crate::wstr!(time_limit.as_ref())))?;
        Ok(())
    }

    pub fn restart_interval(&self) -> crate::task_scheduler::TaskSchedulerResult<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_RestartInterval(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_restart_interval<S: AsRef<str>>(
        &self,
        interval: S,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!((*self.0).put_RestartInterval(crate::wstr!(interval.as_ref())))?;
        Ok(())
    }

    pub fn xml_text(&self) -> crate::task_scheduler::TaskSchedulerResult<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_XmlText(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_xml_text<S: AsRef<str>>(
        &self,
        xml_text: S,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!((*self.0).put_XmlText(crate::wstr!(xml_text.as_ref())))?;
        Ok(())
    }

    pub fn multiple_instances(
        &self,
    ) -> crate::task_scheduler::TaskSchedulerResult<TaskInstancesPolicy> {
        let mut ret: u32 = 0;
        crate::w32_ok!((*self.0).get_MultipleInstances(&mut ret))?;
        Ok(ret.into())
    }

    pub fn set_multiple_instances(
        &self,
        policy: TaskInstancesPolicy,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!((*self.0).put_MultipleInstances(policy as u32))?;
        Ok(())
    }

    pub fn compatibility(&self) -> crate::task_scheduler::TaskSchedulerResult<TaskCompatibility> {
        let mut ret: u32 = 0;
        crate::w32_ok!((*self.0).get_Compatibility(&mut ret))?;
        Ok(ret.into())
    }

    pub fn set_compatibility(
        &self,
        compat: TaskCompatibility,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!((*self.0).put_Compatibility(compat as u32))?;
        Ok(())
    }
}
