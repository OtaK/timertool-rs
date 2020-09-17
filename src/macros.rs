#[macro_export]
macro_rules! w32_ok {
    ($call: expr) => {{
        let result = unsafe { $call };
        if result == winapi::shared::ntstatus::STATUS_SUCCESS {
            Ok(())
        } else {
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

    (BOOL $call: expr) => {{
        let result = unsafe { $call };
        if result == winapi::shared::minwindef::TRUE {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }};
}
