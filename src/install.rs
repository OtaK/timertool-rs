use log::{debug, info};

fn args_to_string(args: Vec<String>) -> String {
    args.into_iter()
        .enumerate()
        .fold(String::new(), |mut s, (i, arg)| {
            if i > 0 {
                s.push(' ');
            }
            s.push_str(&arg);
            s
        })
}

pub fn install(args: &crate::Opts) -> std::io::Result<()> {
    // Copy exe to %ProgramFiles%\TimerSet\TimerSet.exe
    let current_exe_path = std::env::current_exe()?;
    debug!("Current exe path: {:?}", current_exe_path);

    let mut dest_path: std::path::PathBuf = std::env::var("PROGRAMFILES").unwrap().into();
    dest_path.push("TimerSet");
    info!("Installing TimerSet at: {:?}", dest_path);
    debug!("Creating app folder");
    std::fs::create_dir_all(dest_path.clone())?;
    dest_path.push("TimerSet.exe");

    debug!(
        "Moving timerset.exe from {:?} to {:?}",
        current_exe_path, dest_path
    );
    std::fs::copy(current_exe_path, dest_path.clone())?;

    let mut start_args = vec![
        dest_path.to_str().unwrap().into(),
    ];

    if let Some(timer_value) = args.timer {
        start_args.push(format!("--timer {}", timer_value));
    }

    if args.clean_standby_list {
        start_args.push("--islc".to_string());
        if args.clean_standby_list_poll_freq != 10 {
            start_args.push(format!("--islc-timer {}", args.clean_standby_list_poll_freq));
        }
        if args.clear_standby_cached_mem != 1024 {
            start_args.push(format!("--cscm {}", args.clear_standby_cached_mem));
        }
        if args.clear_standby_free_mem != 1024 {
            start_args.push(format!("--csfm {}", args.clear_standby_free_mem));
        }
    }

    // Write registry entry in HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run
    let startup = get_startup_key()?;
    let start_args = args_to_string(start_args);
    debug!("Writing registry value at HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run: TimerSet = {}", start_args);
    startup.set_value("TimerSet", &start_args)?;

    info!("Installation complete.");
    Ok(())
}

pub fn uninstall() -> std::io::Result<()> {
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
        KEY_SET_VALUE,
    )
}

#[cfg(test)]
mod test {
    #[test]
    fn should_format_args_correctly() {
        let test_args = vec![
            "--arg1".to_string(),
            "--arg2".into(),
            "--arg3 withvalue".into(),
        ];

        assert_eq!(super::args_to_string(test_args), "--arg1 --arg2 --arg3 withvalue");
    }
}
