use winapi::{
    shared::wtypes::BSTR,
    um::taskschd::{
        IActionCollection, IPrincipal, IRegistrationInfo, ITaskDefinition, ITaskSettings,
        ITriggerCollection,
    },
};

use super::{
    action::ActionCollection, IUnknownWrapper, Principal, TaskRegistrationInfo, TaskSettings,
    TriggerCollection,
};
pub struct TaskDefinition(IUnknownWrapper<ITaskDefinition>);

impl From<*mut ITaskDefinition> for TaskDefinition {
    fn from(definition: *mut ITaskDefinition) -> Self {
        Self(definition.into())
    }
}

impl Into<*mut ITaskDefinition> for TaskDefinition {
    fn into(self) -> *mut ITaskDefinition {
        self.0.into()
    }
}

#[allow(dead_code)]
impl TaskDefinition {
    pub fn xml_text(&self) -> crate::task_scheduler::TaskSchedulerResult<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!(self.0.get_XmlText(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_xml_text<S: AsRef<str>>(
        &self,
        xml: S,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!(self.0.put_XmlText(crate::wstr!(xml.as_ref())))?;
        Ok(())
    }

    pub fn data(&self) -> crate::task_scheduler::TaskSchedulerResult<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!(self.0.get_Data(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_data<S: AsRef<str>>(
        &self,
        data: S,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        crate::w32_ok!(self.0.put_Data(crate::wstr!(data.as_ref())))?;
        Ok(())
    }

    pub fn principal(&self) -> crate::task_scheduler::TaskSchedulerResult<Principal> {
        let mut principal: *mut IPrincipal = std::ptr::null_mut();
        crate::w32_ok!(self.0.get_Principal(&mut principal as *mut *mut _ as _))?;
        Ok(principal.into())
    }

    pub fn set_principal(
        &self,
        principal: Principal,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        let ptr: *mut IPrincipal = principal.into();
        crate::w32_ok!(self.0.put_Principal(ptr))?;
        Ok(())
    }

    pub fn actions(&self) -> crate::task_scheduler::TaskSchedulerResult<ActionCollection> {
        let mut actions: *mut IActionCollection = std::ptr::null_mut();
        crate::w32_ok!(self.0.get_Actions(&mut actions as *mut *mut _ as _))?;
        Ok(actions.into())
    }

    pub fn set_actions(
        &self,
        actions: ActionCollection,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        let ptr: *mut IActionCollection = actions.into();
        crate::w32_ok!(self.0.put_Actions(ptr))?;
        Ok(())
    }

    pub fn registration_info(
        &self,
    ) -> crate::task_scheduler::TaskSchedulerResult<TaskRegistrationInfo> {
        let mut registration_info: *mut IRegistrationInfo = std::ptr::null_mut();
        crate::w32_ok!(self
            .0
            .get_RegistrationInfo(&mut registration_info as *mut *mut _ as _))?;
        Ok(registration_info.into())
    }

    pub fn set_registration_info(
        &self,
        registration_info: TaskRegistrationInfo,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        let ptr: *mut IRegistrationInfo = registration_info.into();
        crate::w32_ok!(self.0.put_RegistrationInfo(ptr))?;
        Ok(())
    }

    pub fn settings(&self) -> crate::task_scheduler::TaskSchedulerResult<TaskSettings> {
        let mut settings: *mut ITaskSettings = std::ptr::null_mut();
        crate::w32_ok!(self.0.get_Settings(&mut settings as *mut *mut _ as _))?;
        Ok(settings.into())
    }

    pub fn set_settings(
        &self,
        settings: TaskSettings,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        let ptr: *mut ITaskSettings = settings.into();
        crate::w32_ok!(self.0.put_Settings(ptr))?;
        Ok(())
    }

    pub fn triggers(&self) -> crate::task_scheduler::TaskSchedulerResult<TriggerCollection> {
        let mut triggers: *mut ITriggerCollection = std::ptr::null_mut();
        crate::w32_ok!(self.0.get_Triggers(&mut triggers as *mut *mut _ as _))?;
        Ok(triggers.into())
    }

    pub fn set_triggers(
        &self,
        triggers: TriggerCollection,
    ) -> crate::task_scheduler::TaskSchedulerResult<()> {
        let ptr: *mut ITriggerCollection = triggers.into();
        crate::w32_ok!(self.0.put_Triggers(ptr))?;
        Ok(())
    }
}
