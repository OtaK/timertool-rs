#![cfg_attr(not(feature = "enable_log"), windows_subsystem = "windows")]

use log::{error, info};

mod install;
mod standby;
mod timer;
mod win_elevated;
mod macros;

#[derive(Debug, structopt::StructOpt)]
#[structopt(author = "Mathieu Amiot <amiot.mathieu@gmail.com>")]
/// TimerSet allows you to change your NT Kernel system timer
/// Also allows you to monitor Windows Standby List and clean it up when needed
pub struct Opts {
    #[structopt(short, long)]
    /// Installs TimerSet to your system and runs it on startup
    install: bool,

    #[structopt(short, long)]
    /// Uninstalls TimerSet from your system
    uninstall: bool,

    #[structopt(short, long)]
    /// Allows to set a custom timer value in μs. Will be clamped between the bounds of allowed timer values.
    /// Also note that sometimes, setting high timer values are rejected by the system and will be lowered down depending
    /// on which clock source your system is using (TSC tends to lower values by 5μs, HPET does not for instance)
    timer: Option<u32>,

    #[structopt(long = "islc")]
    /// Enables Windows Standby List periodic cleaning.
    /// It is akin to how ISLC by Wagnard works.
    /// Please note that when enabling this, the program will **NOT** be idle at all times and will periodically
    /// poll the system memory to check whether a cleanup is needed or not.
    clean_standby_list: bool,

    #[structopt(long = "islc-timer", default_value = "10")]
    /// Standby List periodic cleaning poll interval.
    /// Defaults to 10 seconds which should be enough for most systems without impacting performance.
    clean_standby_list_poll_freq: u64,

    #[structopt(long = "cscm", default_value = "1024")]
    /// Cached memory threshold where the Windows Standby List will be cleared (in MB)
    /// Defaults to 1024MB (1GB)
    clear_standby_cached_mem: u32,

    #[structopt(long = "csfm", default_value = "1024")]
    /// Free memory threshold where the Windows Standby List will be cleared (in MB)
    /// Defaults to 1024MB (1GB)
    clear_standby_free_mem: u32,

    #[structopt(short, long)]
    /// Prints the possible timer value range for your system.
    /// Please note that it can depend on many factors such as HPET or dynamic/synthetic timers enabled or disabled.
    values: bool,
}

#[cfg(not(windows))]
fn main() {
    panic!("No idea how you compiled this but this software is only compatible with Windows.");
}

#[cfg(windows)]
#[paw::main]
fn main(mut args: Opts) -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    pretty_env_logger::init();
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
            if !win_elevated::is_app_elevated() {
                error!("You need to start this app with administrator permissions to install the program on your system.");
            } else if args.install {
                install::install(&args)?;
            } else if args.uninstall {
                // Revert install steps
                install::uninstall()?;
            }
        }

        timer_info.apply_timer(timer_value)?;
        info!("New timer value set: {}μs", timer_info.cur);
    }

    if args.clean_standby_list {
        if !win_elevated::is_app_elevated() {
            error!("You need to start this app with administrator permissions to use the standby list cleaning feature.");
            return Ok(());
        }

        let cleaner = standby::StandbyListCleaner::default()
            .standby_list_size_threshold(args.clear_standby_cached_mem)
            .free_memory_size_threshold(args.clear_standby_free_mem)
            .poll_interval(args.clean_standby_list_poll_freq);

        drop(args);
        info!("Cleaned up resources and starting memory monitoring...");
        cleaner.monitor_and_clean()?;
    } else {
        drop(args);
        info!("Cleaned up resources and parking till the end of time...");
        std::thread::park();
    }

    Ok(())
}
