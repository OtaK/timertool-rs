use ntapi::ntexapi::{
    NtQuerySystemInformation, NtSetSystemInformation, SystemMemoryListInformation,
    SYSTEM_MEMORY_LIST_INFORMATION,
};

use winapi::um::{
    handleapi::CloseHandle,
    processthreadsapi::{GetCurrentProcess, OpenProcessToken},
    securitybaseapi::AdjustTokenPrivileges,
    sysinfoapi::{GetSystemInfo, SYSTEM_INFO},
    winbase::LookupPrivilegeValueA,
    winnt::{SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY},
};

use log::{debug, info};

#[derive(Copy, Clone)]
#[repr(transparent)]
struct SystemMemoryListInformationWrapper(SYSTEM_MEMORY_LIST_INFORMATION);

impl std::fmt::Debug for SystemMemoryListInformationWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemMemoryListInformation")
            .field("ZeroPageCount", &self.0.ZeroPageCount)
            .field("FreePageCount", &self.0.FreePageCount)
            .field("ModifiedPageCount", &self.0.ModifiedPageCount)
            .field("BadPageCount", &self.0.BadPageCount)
            .field("PageCountByPriority", &self.0.PageCountByPriority)
            .field(
                "RepurposedPagesByPriority",
                &self.0.RepurposedPagesByPriority,
            )
            .field(
                "ModifiedPageCountPageFile",
                &self.0.ModifiedPageCountPageFile,
            )
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct StandbyListCleaner {
    standbylist_threshold: usize,
    freemem_threshold: usize,
    poll_freq: std::time::Duration,
}

impl Default for StandbyListCleaner {
    fn default() -> Self {
        Self {
            standbylist_threshold: 1024,
            freemem_threshold: 1024,
            poll_freq: std::time::Duration::from_secs(10),
        }
    }
}

impl StandbyListCleaner {
    /// Standby List size threshold in MB
    pub fn standby_list_size_threshold(mut self, threshold: u32) -> Self {
        self.standbylist_threshold = threshold as usize * 1_000_000;
        self
    }

    /// Free memory threshold in MB
    pub fn free_memory_size_threshold(mut self, threshold: u32) -> Self {
        self.freemem_threshold = threshold as usize * 1_000_000;
        self
    }

    /// Configurable poll interval in seconds
    pub fn poll_interval(mut self, interval: u64) -> Self {
        self.poll_freq = std::time::Duration::from_secs(interval);
        self
    }

    unsafe fn upgrade_security_token(&self) -> std::io::Result<()> {
        debug!("Beginning to upgrade security token...");
        let process_hwnd = GetCurrentProcess();
        let mut token_hwnd = winapi::shared::ntdef::NULL;
        let result = OpenProcessToken(
            process_hwnd,
            TOKEN_QUERY | TOKEN_ADJUST_PRIVILEGES,
            &mut token_hwnd,
        );
        if result == winapi::shared::minwindef::FALSE {
            return Err(std::io::Error::last_os_error());
        }

        let mut luid = winapi::shared::ntdef::LUID::default();
        let lp_name =
            std::ffi::CStr::from_bytes_with_nul_unchecked(b"SeProfileSingleProcessPrivilege\0");
        let result = LookupPrivilegeValueA(0 as _, lp_name.as_ptr() as _, &mut luid as _);
        if result == winapi::shared::minwindef::FALSE {
            return Err(std::io::Error::last_os_error());
        }

        debug!(
            "LookupPrivilegeValueA returned LUID Low = {:x} / High = {:x}",
            luid.LowPart, luid.HighPart
        );

        let mut new_privileges = TOKEN_PRIVILEGES::default();
        let mut old_privileges = TOKEN_PRIVILEGES::default();
        let mut dw_buffer_length = 16u32;

        new_privileges.PrivilegeCount = 1;
        new_privileges.Privileges[0].Luid = luid;
        new_privileges.Privileges[0].Attributes = 0;
        let result = AdjustTokenPrivileges(
            token_hwnd,
            0,
            &mut new_privileges as _,
            std::mem::size_of_val(&new_privileges) as _,
            &mut old_privileges as _,
            &mut dw_buffer_length as _,
        );

        if result == winapi::shared::minwindef::FALSE {
            return Err(std::io::Error::last_os_error());
        }

        debug!("Assigned new privileges successfully");

        old_privileges.PrivilegeCount = 1;
        old_privileges.Privileges[0].Luid = luid;
        old_privileges.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;
        let result = AdjustTokenPrivileges(
            token_hwnd,
            0,
            &mut old_privileges,
            dw_buffer_length,
            winapi::shared::ntdef::NULL as _,
            0 as _,
        );

        if result == winapi::shared::minwindef::FALSE {
            return Err(std::io::Error::last_os_error());
        }

        debug!("Assigned old privileges successfully");

        if CloseHandle(token_hwnd) == winapi::shared::minwindef::FALSE {
            return Err(std::io::Error::last_os_error());
        }

        if CloseHandle(process_hwnd) == winapi::shared::minwindef::FALSE {
            return Err(std::io::Error::last_os_error());
        }

        debug!("Closed process & token handles successfully");

        Ok(())
    }

    /// Starts the monitoring loop.
    /// Note that this is a blocking function that will not exit unless there's an error.
    pub fn monitor_and_clean(&self) -> std::io::Result<()> {
        unsafe { self.upgrade_security_token()? };

        let mut command = ntapi::ntexapi::MemoryPurgeStandbyList;
        let cmd_len = std::mem::size_of_val(&command) as u32;
        let cmd_ptr: *mut u32 = &mut command as _;
        let mut ret_len = 0u32;

        let mut system_info = SYSTEM_INFO::default();
        unsafe {
            GetSystemInfo(&mut system_info as _);
        }
        debug!("System page size is {}", system_info.dwPageSize);

        let page_size = system_info.dwPageSize as usize;

        let mut system_information: SYSTEM_MEMORY_LIST_INFORMATION = unsafe { std::mem::zeroed() };

        loop {
            debug!("Calling NtQuerySystemInformation...");
            let result = unsafe {
                NtQuerySystemInformation(
                    SystemMemoryListInformation,
                    &mut system_information as *mut SYSTEM_MEMORY_LIST_INFORMATION as _,
                    std::mem::size_of::<SYSTEM_MEMORY_LIST_INFORMATION>() as _,
                    &mut ret_len as _,
                )
            };

            debug!(
                "NtQuerySystemInformation(\n{}, \n{:?}, \n{}, \n{}\n) -> {}",
                SystemMemoryListInformation,
                SystemMemoryListInformationWrapper(system_information),
                std::mem::size_of::<SYSTEM_MEMORY_LIST_INFORMATION>(),
                ret_len,
                result
            );

            if result != winapi::shared::ntstatus::STATUS_SUCCESS {
                return Err(std::io::Error::last_os_error());
            }

            let list_mem = system_information.PageCountByPriority.iter().sum::<usize>() * page_size;
            let free_mem = system_information.ZeroPageCount * page_size
                + system_information.FreePageCount * page_size;

            debug!("Free memory: {:.2}MB", free_mem / 1_000_000);
            debug!("Standby List memory: {:.2}MB", list_mem / 1_000_000);

            if free_mem < self.freemem_threshold && list_mem > self.standbylist_threshold {
                info!("Conditions met, now freeing standby list");
                let result = unsafe {
                    NtSetSystemInformation(SystemMemoryListInformation, cmd_ptr as _, cmd_len)
                };

                debug!(
                    "NtSetSystemInformation({}, {:?}, {}) -> {}",
                    SystemMemoryListInformation, cmd_ptr, cmd_len, result
                );

                if result == winapi::shared::ntstatus::STATUS_PRIVILEGE_NOT_HELD {
                    debug!("Lacking admin token to do such an action");
                }

                if result != winapi::shared::ntstatus::STATUS_SUCCESS {
                    return Err(std::io::Error::last_os_error());
                }

                debug!("Standby list cleaned up");
            }

            debug!("Sleeping {} seconds", self.poll_freq.as_secs());
            system_information = unsafe { std::mem::zeroed() };
            std::thread::sleep(self.poll_freq);
        }
    }
}
