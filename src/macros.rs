#[macro_export]
macro_rules! w32_ok {
    ($call:expr) => {{
        let result = unsafe { $call };
        if result == winapi::shared::ntstatus::STATUS_SUCCESS {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }};

    ($call:expr, ELSE $else_call:expr) => {{
        let result = unsafe { $call };
        if result == winapi::shared::ntstatus::STATUS_SUCCESS {
            Ok(())
        } else {
            unsafe { $else_call };
            Err(std::io::Error::last_os_error())
        }
    }};

    (DEBUG $call:expr, $debug_call:expr) => {{
        let result = unsafe { $call };
        $debug_call(result);
        if result == winapi::shared::ntstatus::STATUS_SUCCESS {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }};

    (DEBUG $call:expr, $debug_call:expr, ELSE $else_call:expr) => {{
        match crate::w32_ok!(DEBUG $call, $debug_call) {
            Ok(a) => Ok(a),
            Err(e) => {
                unsafe { $else_call };
                Err(e)
            }
        }
    }};

    (BOOL $call: expr) => {{
        let result = unsafe { $call };
        if result == winapi::shared::minwindef::TRUE {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }};
}

#[macro_export]
macro_rules! wstr {
    ($str:expr) => {{
        use std::os::windows::ffi::OsStrExt as _;
        #[allow(unused_unsafe)]
        unsafe {
            winapi::um::oleauto::SysAllocString(
                std::ffi::OsStr::new(&format!("{}\0", $str))
                    .encode_wide()
                    .collect::<Vec<u16>>()
                    .as_mut_ptr(),
            )
        }
    }};
}

#[macro_export]
macro_rules! bstr_variant {
    ($bstr:ident) => {{
        let mut variant: winapi::um::oaidl::VARIANTARG = unsafe { std::mem::zeroed() };
        unsafe { winapi::um::oleauto::VariantInit(&mut variant as *mut _ as _) };
        let mut var_n2 = unsafe { variant.n1.n2_mut() };
        var_n2.vt = winapi::shared::wtypes::VT_BSTR as u16;
        let val_ptr = unsafe { var_n2.n3.bstrVal_mut() };
        *val_ptr = $bstr;
        variant
    }};
}
