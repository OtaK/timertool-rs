use winapi::um::taskschd::IRegisteredTask;

use super::IUnknownWrapper;
pub struct RegisteredTask(IUnknownWrapper<IRegisteredTask>);

impl From<*mut IRegisteredTask> for RegisteredTask {
    fn from(definition: *mut IRegisteredTask) -> Self {
        Self(definition.into())
    }
}
