use ntapi::ntexapi::{
    SYSTEM_MEMORY_LIST_INFORMATION,
    SystemMemoryListInformation,
    NtQuerySystemInformation,
    NtSetSystemInformation,
};


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

    /// Starts the monitoring loop.
    /// Note that this is a blocking function that will not exit unless there's an error.
    pub fn monitor_and_clean(&self) -> std::io::Result<()> {
        let mut commands = [
            ntapi::ntexapi::MemoryPurgeStandbyList
        ];
        let cmd_len = std::mem::size_of_val(&commands) as u32;
        let cmd_ptr = commands.as_mut_ptr() as _;

        let mut system_information: SYSTEM_MEMORY_LIST_INFORMATION = unsafe { std::mem::zeroed() };

        loop {
            let result = unsafe {
                NtQuerySystemInformation(
                    SystemMemoryListInformation,
                    &mut system_information as *mut SYSTEM_MEMORY_LIST_INFORMATION as _,
                    std::mem::size_of::<SYSTEM_MEMORY_LIST_INFORMATION>() as _,
                    winapi::shared::ntdef::NULL as _
                )
            };

            if result != winapi::shared::ntstatus::STATUS_SUCCESS {
                return Err(std::io::Error::last_os_error());
            }

            let free_mem = system_information.FreePageCount;
            let list_mem: usize = system_information.PageCountByPriority.iter().sum();

            if free_mem < self.freemem_threshold || list_mem > self.standbylist_threshold {
                let result = unsafe {
                    NtSetSystemInformation(
                        SystemMemoryListInformation,
                        cmd_ptr,
                        cmd_len,
                    )
                };

                if result != winapi::shared::ntstatus::STATUS_SUCCESS {
                    return Err(std::io::Error::last_os_error());
                }
            }

            std::thread::sleep(self.poll_freq);
        }
    }
}
