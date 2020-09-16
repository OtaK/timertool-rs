use log::{debug, info};

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

    debug!("Moving timerset.exe from {:?} to {:?}", current_exe_path, dest_path);
    std::fs::copy(current_exe_path, dest_path.clone())?;

    let mut start_args: String = dest_path.to_str().unwrap().into();

    if let Some(timer_value) = args.timer {
        start_args.push_str(&format!(" --timer {}", timer_value));
    }

    // Write registry entry in HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run
    let startup = get_startup_key()?;
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
        KEY_SET_VALUE
    )
}
