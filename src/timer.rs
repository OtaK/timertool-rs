#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct TimerResolutionInfo {
    pub cur: u32,
    pub min: u32,
    pub max: u32,
}

impl std::fmt::Display for TimerResolutionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "System Timer Values: min [{}μs] / max [{}μs] / cur [{}μs]",
            self.min, self.max, self.cur
        )
    }
}

impl TimerResolutionInfo {
    pub fn fetch() -> std::io::Result<Self> {
        let mut min = 0u32;
        let mut max = 0u32;
        let mut cur = 0u32;
        let status =
            unsafe { ntapi::ntexapi::NtQueryTimerResolution(&mut min, &mut max, &mut cur) };

        if status == winapi::shared::ntstatus::STATUS_SUCCESS {
            Ok(Self { min, max, cur })
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    pub fn apply_timer(&mut self, value: u32) -> std::io::Result<()> {
        let value = self.clamp_timer_value(value);
        let status = unsafe { ntapi::ntexapi::NtSetTimerResolution(value, 1, &mut self.cur) };

        if status != winapi::shared::ntstatus::STATUS_SUCCESS {
            return Err(std::io::Error::last_os_error());
        }

        Ok(())
    }

    pub fn clamp_timer_value(&self, value: u32) -> u32 {
        if value > self.min {
            self.min
        } else if value < self.max {
            self.max
        } else {
            value
        }
    }
}
