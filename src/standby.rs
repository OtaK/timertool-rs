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

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct StandbyListCleaner {
    standbylist_threshold: usize,
    freemem_threshold: usize,
    poll_freq: std::time::Duration,
    #[cfg(feature = "cmrn")]
    memory_hwnd: winapi::shared::ntdef::HANDLE,
    #[cfg(feature = "cmrn")]
    last_memory_wait: Option<std::time::Instant>,
}

impl Default for StandbyListCleaner {
    fn default() -> Self {
        Self {
            standbylist_threshold: 1024,
            freemem_threshold: 1024,
            poll_freq: std::time::Duration::from_secs(10),
            #[cfg(feature = "cmrn")]
            memory_hwnd: winapi::shared::ntdef::NULL,
            #[cfg(feature = "cmrn")]
            last_memory_wait: None,
        }
    }
}

#[cfg(feature = "cmrn")]
impl Drop for StandbyListCleaner {
    fn drop(&mut self) {
        let _ = self.cleanup_cmrn();
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

    #[cfg(feature = "cmrn")]
    fn setup_cmrn(&mut self) -> std::io::Result<()> {
        let mm_reg = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE).open_subkey_with_flags(
            "System\\CurrentControlSet\\Control\\Session Manager\\Memory Management",
            winreg::enums::KEY_WRITE
        )?;

        mm_reg.set_value("LowMemoryThreshold", &(self.freemem_threshold as u32))?;

        self.memory_hwnd = unsafe {
            winapi::um::memoryapi::CreateMemoryResourceNotification(
                winapi::um::memoryapi::LowMemoryResourceNotification
            )
        };

        Ok(())
    }

    #[cfg(feature = "cmrn")]
    fn cleanup_cmrn(&mut self) -> std::io::Result<()> {
        debug!("Cleaning up memory handle");
        unsafe { CloseHandle(self.memory_hwnd) };

        debug!("Cleaning up LowMemoryThreshold registry key...");
        let mm_reg = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE).open_subkey_with_flags(
            "System\\CurrentControlSet\\Control\\Session Manager\\Memory Management",
            winreg::enums::KEY_WRITE
        )?;
        mm_reg.delete_value("LowMemoryThreshold")?;
        Ok(())
    }

    fn upgrade_security_token(&self) -> std::io::Result<()> {
        debug!("Beginning to upgrade security token...");
        let process_hwnd = unsafe { GetCurrentProcess() };
        let mut token_hwnd = winapi::shared::ntdef::NULL;
        crate::w32_ok!(BOOL OpenProcessToken(
            process_hwnd,
            TOKEN_QUERY | TOKEN_ADJUST_PRIVILEGES,
            &mut token_hwnd,
        ))?;

        let mut luid = winapi::shared::ntdef::LUID::default();
        let lp_name = unsafe {
            std::ffi::CStr::from_bytes_with_nul_unchecked(b"SeProfileSingleProcessPrivilege\0")
        };
        crate::w32_ok!(BOOL LookupPrivilegeValueA(0 as _, lp_name.as_ptr() as _, &mut luid as _))?;

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
        crate::w32_ok!(BOOL AdjustTokenPrivileges(
            token_hwnd,
            0,
            &mut new_privileges as _,
            std::mem::size_of_val(&new_privileges) as _,
            &mut old_privileges as _,
            &mut dw_buffer_length as _,
        ))?;

        debug!("Assigned new privileges successfully");

        old_privileges.PrivilegeCount = 1;
        old_privileges.Privileges[0].Luid = luid;
        old_privileges.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;
        crate::w32_ok!(BOOL AdjustTokenPrivileges(
            token_hwnd,
            0,
            &mut old_privileges,
            dw_buffer_length,
            winapi::shared::ntdef::NULL as _,
            0 as _,
        ))?;

        debug!("Assigned old privileges successfully");

        crate::w32_ok!(BOOL CloseHandle(token_hwnd))?;
        crate::w32_ok!(BOOL CloseHandle(process_hwnd))?;
        debug!("Closed process & token handles successfully");

        Ok(())
    }

    /// Starts the monitoring loop.
    /// Note that this is a blocking function that will not exit unless there's an error.
    pub fn monitor_and_clean(&mut self) -> std::io::Result<()> {
        self.upgrade_security_token()?;
        #[cfg(feature = "cmrn")]
        self.setup_cmrn()?;

        let mut command = ntapi::ntexapi::MemoryPurgeStandbyList;
        let cmd_len = std::mem::size_of_val(&command) as u32;
        let cmd_ptr: *mut u32 = &mut command as _;
        let mut ret_len = 0u32;

        let mut system_info = SYSTEM_INFO::default();
        unsafe { GetSystemInfo(&mut system_info as _) };
        debug!("System page size is {}", system_info.dwPageSize);

        let page_size = system_info.dwPageSize as usize;

        let mut system_information: SYSTEM_MEMORY_LIST_INFORMATION = unsafe { std::mem::zeroed() };

        loop {
            debug!("Calling NtQuerySystemInformation...");
            // Calling NtQuerySystemInformation with the undocumented SystemMemoryListInformation parameter allows
            // to retrieve the stats of cached/freed/zeroed pages.
            crate::w32_ok!(DEBUG
                NtQuerySystemInformation(
                    SystemMemoryListInformation,
                    &mut system_information as *mut SYSTEM_MEMORY_LIST_INFORMATION as _,
                    std::mem::size_of::<SYSTEM_MEMORY_LIST_INFORMATION>() as _,
                    &mut ret_len as _,
                ),
                |result| debug!(
                    "NtQuerySystemInformation(\n{}, \n{:?}, \n{}, \n{}\n) -> {}",
                    SystemMemoryListInformation,
                    SystemMemoryListInformationWrapper(system_information),
                    std::mem::size_of::<SYSTEM_MEMORY_LIST_INFORMATION>(),
                    ret_len,
                    result
                )
            )?;

            // Undocumented: StandbyList size is calculated by summing all the page count per priority
            // (and multiplying by the page size -usually 4KB- to get the value in bytes)
            let list_mem = system_information.PageCountByPriority.iter().sum::<usize>() * page_size;
            // Undocumented: Free memory is the sum of zeroed AND free pages
            // Free memory here is actual free, zeroed, non-repurposed physical memory
            let free_mem = system_information.ZeroPageCount * page_size
                + system_information.FreePageCount * page_size;

            debug!("Free memory: {:.2}MB", free_mem / 1_000_000);
            debug!("Standby List memory: {:.2}MB", list_mem / 1_000_000);

            if free_mem < self.freemem_threshold && list_mem > self.standbylist_threshold {
                info!("Conditions met, now freeing standby list");

                // Calling NtSetSystemInformation with the undocumented MemoryPurgeStandbyList command triggers
                // purging the StandbyList, allowing to reclaim cached physical memory as free.
                // This command is usually blocking for a few seconds since the kernel call blocks until
                // all of the standby list is freed
                crate::w32_ok!(DEBUG
                    NtSetSystemInformation(SystemMemoryListInformation, cmd_ptr as _, cmd_len),
                    |result| {
                        debug!(
                            "NtSetSystemInformation({}, {:?}, {}) -> {}",
                            SystemMemoryListInformation, cmd_ptr, cmd_len, result
                        );

                        if result == winapi::shared::ntstatus::STATUS_PRIVILEGE_NOT_HELD {
                            debug!("Lacking admin token to do such an action");
                        }
                    }
                )?;

                debug!("Standby list cleaned up");
            }

            if cfg!(feature = "cmrn") {
                debug!("Waiting on memory notification now...");
            } else {
                debug!("Sleeping {} seconds", self.poll_freq.as_secs());
            }

            // TODO: Switch to CreateMemoryResourceNotification?
            // This could completely eliminate the need to periodically poll and might be way way more efficient in the long run
            // cf. https://forums.guru3d.com/threads/fix-game-stutter-on-win-10-1703-1809.420251/page-12#post-5590984
            self.wait_next_iter()?;
        }
    }

    #[cfg(not(feature = "cmrn"))]
    fn wait_next_iter(&mut self) -> std::io::Result<()> {
        std::thread::sleep(self.poll_freq);
        Ok(())
    }

    #[cfg(feature = "cmrn")]
    fn wait_next_iter(&mut self) -> std::io::Result<()> {
        if let Some(elapsed) = self.last_memory_wait.as_ref().map(std::time::Instant::elapsed) {
            if elapsed < self.poll_freq {
                let sleep_dur = self.poll_freq - elapsed;
                debug!("Anti Kernel-DOS triggered, sleeping {}s", sleep_dur.as_secs());
                std::thread::sleep(sleep_dur);
            }
        }

        use winapi::um::synchapi::WaitForSingleObject;
        use winapi::um::winbase::{
            WAIT_ABANDONED,
            WAIT_FAILED,
            WAIT_OBJECT_0,
        };
        self.last_memory_wait = Some(std::time::Instant::now());
        let result = unsafe { WaitForSingleObject(self.memory_hwnd, winapi::um::winbase::INFINITE) };
        match result {
            WAIT_FAILED => Err(std::io::Error::last_os_error()),
            WAIT_OBJECT_0 | WAIT_ABANDONED => Ok(()),
            _ => unreachable!(),
        }
    }
}
