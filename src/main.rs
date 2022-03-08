#![cfg_attr(
    all(not(feature = "loggity-log"), not(debug_assertions)),
    windows_subsystem = "windows"
)]

use log::{error, info};

mod utils;

mod error;
mod install;
mod logger;
mod macros;
mod standby;
mod task_scheduler;
mod timer;
pub use self::error::*;

/// TimerSet allows you to change your NT Kernel system timer
/// Also allows you to monitor Windows Standby List and clean it up when needed
#[derive(Debug, clap::Parser)]
#[clap(about, version, author)]
pub struct Opts {
    /// Shows the actions taken but do not modify anything on the system; Also known as a dry run.
    #[clap(short, long)]
    pretend: bool,

    /// Installs TimerSet to your system and runs it on startup
    #[clap(short, long)]
    install: bool,

    /// Uninstalls TimerSet from your system
    #[clap(short, long)]
    uninstall: bool,

    /// Allows to set a custom timer value in μs. Will be clamped between the bounds of allowed timer values.
    /// Also note that sometimes, setting high timer values are rejected by the system and will be lowered down depending
    /// on which clock source your system is using (TSC tends to lower values by ~5μs, HPET does not for instance)
    #[clap(short, long)]
    timer: Option<u32>,

    /// Enables Windows Standby List periodic cleaning.
    /// It is akin to how ISLC by Wagnard works.
    #[clap(long = "islc")]
    clean_standby_list: bool,

    /// Standby List anti-kernel DOS throttle timer
    /// It exists because CreateMemoryResourceNotification can trigger LowMemoryResourceNotifications
    /// thousands of times per second when they happen (i.e. every system page allocation in a high memory pressure situation, often 4KB)
    /// resulting in the memory list cleaning paralyzing the system with thousands of tries per second
    ///
    /// Defaults to 10 seconds which should be enough for most systems without impacting performance.
    #[clap(long = "islc-timer", default_value = "10")]
    clean_standby_list_poll_freq: u64,

    /// Cached memory threshold where the Windows Standby List will be cleared (in MB)
    /// Defaults to 1024MB (1GB)
    #[clap(long = "cscm", default_value = "1024")]
    clear_standby_cached_mem: u32,

    /// Free memory threshold where the Windows Standby List will be cleared (in MB)
    /// Defaults to 1024MB (1GB)
    #[clap(long = "csfm", default_value = "1024")]
    clear_standby_free_mem: u32,

    /// Prints the possible timer value range for your system.
    /// Please note that it can depend on many factors such as HPET or dynamic/synthetic timers enabled or disabled.
    #[clap(short, long)]
    values: bool,
}

#[cfg(not(windows))]
fn main() {
    panic!("No idea how you compiled this but this software is only compatible with Windows.");
}

#[cfg(windows)]
fn main() -> TimersetResult<()> {
    let mut logger = logger::Logger::new();
    logger.init()?;

    use clap::Parser as _;
    let mut args = Opts::parse();

    if args.pretend {
        info!("--pretend enabled, no action will be taken on the system")
    }

    {
        let mut timer_info = timer::TimerResolutionInfo::fetch()?;
        info!("{}", timer_info);

        if args.values {
            return Ok(());
        }

        if let Some(timer) = args.timer.as_mut() {
            *timer = timer_info.clamp_timer_value(*timer);
        }

        let timer_value = args.timer.unwrap_or(timer_info.max);

        info!("Chosen timer value: {}μs", timer_value);

        if args.install || args.uninstall {
            if !utils::win_elevated::is_app_elevated() {
                error!("You need to start this app with administrator permissions to install the program on your system.");
            } else if args.install {
                install::install(&args)?;
            } else if args.uninstall {
                // Revert install steps
                install::uninstall(&args)?;
            }

            return Ok(());
        }

        if !args.pretend {
            timer_info.apply_timer(timer_value)?;
        }
        info!("New timer value set: {}μs", timer_info.cur);
    }

    if args.clean_standby_list {
        if !utils::win_elevated::is_app_elevated() {
            error!("You need to start this app with administrator permissions to use the standby list cleaning feature.");
            return Ok(());
        }

        let mut cleaner = standby::StandbyListCleaner::default()
            .standby_list_size_threshold(args.clear_standby_cached_mem)
            .free_memory_size_threshold(args.clear_standby_free_mem)
            .poll_interval(args.clean_standby_list_poll_freq);

        if args.pretend {
            return Ok(());
        }
        drop(args);
        info!("Cleaned up resources and starting memory monitoring...");
        cleaner.monitor_and_clean()?;
    } else {
        if args.pretend {
            return Ok(());
        }
        drop(args);
        info!("Cleaned up resources and parking till the end of time...");
        loop {
            std::thread::park();
        }
    }

    Ok(())
}
