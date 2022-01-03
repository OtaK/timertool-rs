#![allow(clippy::from_over_into)]

mod error;
pub use error::*;

mod task_folder;
pub use task_folder::*;

mod task_settings;
pub use task_settings::*;

mod task_registration_info;
pub use task_registration_info::*;

mod task_definition;
pub use task_definition::*;

mod principal;
pub use principal::*;

mod trigger;
pub use trigger::*;

mod task_service;
pub use task_service::*;

mod action;
pub use action::*;

mod registered_task;
pub use registered_task::*;

use std::ops::Deref;
use winapi::um::oaidl::IDispatch;

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct IUnknownWrapper<T: Deref<Target = IDispatch>>(Option<Box<T>>);

impl<T: Deref<Target = IDispatch>> Into<*mut T> for IUnknownWrapper<T> {
    fn into(mut self) -> *mut T {
        self.0
            .take()
            .or_else(|| panic!("Empty IUnknownWrapper, shouldn't happen"))
            .map(Box::into_raw)
            .unwrap()
    }
}

impl<T: Deref<Target = IDispatch>> From<*mut T> for IUnknownWrapper<T> {
    fn from(ptr: *mut T) -> Self {
        Self(Some(unsafe { Box::from_raw(ptr) }))
    }
}

impl<T: Deref<Target = IDispatch>> Drop for IUnknownWrapper<T> {
    fn drop(&mut self) {
        if let Some(bptr) = self.0.take() {
            unsafe {
                (*bptr).Release();
                let ptr = Box::into_raw(bptr);
                std::ptr::drop_in_place(ptr);
            }
        }
    }
}

impl<T: Deref<Target = IDispatch>> std::ops::Deref for IUnknownWrapper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        if let Some(b) = &self.0 {
            b.as_ref()
        } else {
            panic!("Empty IUnknownWrapper, shouldn't happen")
        }
    }
}

impl<T: Deref<Target = IDispatch>> std::ops::DerefMut for IUnknownWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Some(b) = &mut self.0 {
            b.as_mut()
        } else {
            panic!("Empty IUnknownWrapper, shouldn't happen")
        }
    }
}

#[inline(always)]
pub(crate) fn bstr_to_string(
    bstr: winapi::shared::wtypes::BSTR,
) -> crate::task_scheduler::TaskSchedulerResult<String> {
    let raw_str = unsafe { std::slice::from_raw_parts(bstr, *bstr as _) };
    use std::os::windows::ffi::OsStringExt as _;
    let os_str = std::ffi::OsString::from_wide(raw_str);
    Ok(os_str
        .into_string()
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidData))?)
}
