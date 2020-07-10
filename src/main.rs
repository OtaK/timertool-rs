#![windows_subsystem = "windows"]

use log::{error, debug, info};

mod win_elevated;

#[derive(Debug, structopt::StructOpt)]
#[structopt(author = "Mathieu Amiot <amiot.mathieu@gmail.com>")]
/// TimerSet allows you to change your NT Kernel system timer
struct Opts {
    #[structopt(short, long)]
    /// Installs TimerSet to your system and runs it on startup
    install: bool,

    #[structopt(short, long)]
    /// Uninstalls TimerSet from your system
    uninstall: bool,

    #[structopt(short, long)]
    /// Allows to set a custom timer value in μs. Will be clamped between the bounds of allowed timer values.
    /// Also note that sometimes, setting high timer values are rejected by the system and will be lowered down according to unknown factors (windows ancient magic????)
    timer: Option<u32>,

    #[structopt(short, long)]
    /// Prints the possible timer value range for your system.
    /// Please note that it can depend on many factors such as HPET or dynamic/synthetic timers enabled or disabled.
    values: bool,
}


#[cfg(not(windows))]
fn main() {
    panic!("This software is only compatible with Windows.");
}

#[cfg(windows)]
#[paw::main]
fn main(args: Opts) -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    pretty_env_logger::init();
    {
        let mut min_res = 0u32;
        let mut max_res = 0u32;
        let mut cur_res = 0u32;
        unsafe {
            ntapi::ntexapi::NtQueryTimerResolution(&mut min_res, &mut max_res, &mut cur_res);
        }

        info!("System Timer Values: min [{}μs] / max [{}μs] / cur [{}μs]", min_res, max_res, cur_res);

        if args.values {
            return Ok(());
        }

        let timer_value = if let Some(timer) = args.timer {
            if timer > min_res {
                min_res
            } else if timer < max_res {
                max_res
            } else {
                timer
            }
        } else {
            max_res
        };

        info!("Chosen timer value: {}μs", timer_value);

        if args.install || args.uninstall {
            if !win_elevated::is_app_elevated() {
                error!("You need to start this app with administrator permissions to install the program on your system.");
            } else {
                if args.install {
                    install(args.timer.map(|_| {
                        timer_value
                    }))?;
                } else if args.uninstall { // Revert install steps
                    uninstall()?;
                }
            }
        }

        drop(args);

        unsafe {
            ntapi::ntexapi::NtSetTimerResolution(timer_value, 1, &mut cur_res);
        }

        info!("New timer value set: {}μs", cur_res);
    }
    info!("Cleaning up resources and parking till the end of time...");
    std::thread::park();

    Ok(())
}

fn install(timer_value: Option<u32>) -> std::io::Result<()> {
    // Copy exe to %ProgramFiles%\TimerSet\TimerSet.exe
    let current_exe_path = std::env::current_exe()?;
    debug!("Current exe path: {:?}", current_exe_path);

    let mut dest_path: std::path::PathBuf = std::env::var("PROGRAMFILES").unwrap().into();
    dest_path.push("TimerSet");
    info!("Installing TimerSet at: {:?}", dest_path);
    debug!("Creating app folder");
    std::fs::create_dir_all(dest_path.clone())?;
    dest_path.push("TimerSet.exe");

    debug!("Moving timerset.exe from {:?} to {:?}", current_exe_path, dest_path);
    std::fs::copy(current_exe_path, dest_path.clone())?;

    let mut start_args: String = dest_path.to_str().unwrap().into();

    if let Some(timer_value) = timer_value {
        start_args.push_str(&format!(" --timer {}", timer_value));
    }

    // Write registry entry in HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run
    let startup = get_startup_key()?;
    debug!("Writing registry value at HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run: TimerSet = {}", start_args);
    startup.set_value("TimerSet", &start_args)?;

    info!("Installation complete.");
    Ok(())
}

fn uninstall() -> std::io::Result<()> {
    // Unload from boot
    let startup = get_startup_key()?;
    let _ = startup.delete_value("TimerSet"); // Ignore errors since we don't care that it's been removed already

    // Delete files
    let mut dest_path: std::path::PathBuf = std::env::var("PROGRAMFILES").unwrap().into();
    dest_path.push("TimerSet");
    debug!("Installation path to be removed: {:?}", dest_path);
    let _ = std::fs::remove_dir_all(dest_path);

    info!("Uninstall complete.");

    Ok(())
}

fn get_startup_key() -> std::io::Result<winreg::RegKey> {
    use winreg::enums::*;
    let hklm = winreg::RegKey::predef(HKEY_LOCAL_MACHINE);
    hklm.open_subkey_with_flags(
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
        KEY_SET_VALUE
    )
}
