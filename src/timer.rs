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
        // NtQueryTimerResolution is an old, undocumented internal NT Kernel API that allows to retrieve
        // the kernel's timer resolutions. It will give out minimum and maximum resolutions, and the current one.
        crate::w32_ok!(ntapi::ntexapi::NtQueryTimerResolution(&mut min, &mut max, &mut cur))?;
        Ok(Self { min, max, cur })
    }

    pub fn apply_timer(&mut self, value: u32) -> std::io::Result<()> {
        let value = self.clamp_timer_value(value);
        // NtSetTimerResolution is an old, undocumented internal NT Kernel API that is very often used by media applications
        // to raise the kernel's timer and allow lower latencies and higher (= closer to real-time) throughput
        // This call sets the desired timer resolution and keeps it effective as long as the calling process is running.
        // Once the application exits, the timer will be set at the lowest-requested timer by any app on the system, or if no app
        // requests a specific timer resolution, it'll be reset at the maximum timer value / lowest resolution to save energy.
        // Also note that this DOES have an effect on latency and throughput, meaning that the myth of a tickless NT Kernel since Windows 8
        // is essentially a lie, for the simple fact that many legacy moving parts of the kernel are still relying on the NT Timer.
        // WDDM is a good example of this for instance, as using a low timer will reduce DPC/ISR latencies for most -if not all- drivers.
        crate::w32_ok!(ntapi::ntexapi::NtSetTimerResolution(value, 1, &mut self.cur))?;
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
