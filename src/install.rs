use crate::task_scheduler::{ExecAction, WindowsTaskScheduler};
use crate::{
    task_scheduler::{
        LogonTrigger,
        TaskActionType,
        TaskCompatibility,
        TaskInstancesPolicy,
        TaskTriggerType,
        TaskLogonType,
        TaskRunlevel
    },
    utils::StartArgs,
};
use log::{debug, info};
use winapi::um::taskschd::TASK_CREATE_OR_UPDATE;

#[cfg(debug)]
const TASK_NAME: &str = "Start TimerSet [DEV]";
#[cfg(not(debug))]
const TASK_NAME: &str = "Start TimerSet";

pub fn install(args: &crate::Opts) -> std::io::Result<()> {
    // Copy exe to %ProgramFiles%\TimerSet\TimerSet.exe
    let current_exe_path = std::env::current_exe()?;
    debug!("Current exe path: {:?}", current_exe_path);

    let mut dest_path: std::path::PathBuf = std::env::var("PROGRAMFILES").unwrap().into();
    dest_path.push("TimerSet");
    info!("Installing TimerSet at: {:?}", dest_path);
    debug!("Creating app folder");
    if !args.pretend {
        std::fs::create_dir_all(dest_path.clone())?;
    }
    dest_path.push("TimerSet.exe");

    debug!(
        "Moving timerset.exe from {:?} to {:?}",
        current_exe_path, dest_path
    );
    if !args.pretend {
        std::fs::copy(current_exe_path, dest_path.clone())?;
    }

    let start_args = StartArgs::build_from_args(dest_path, args);

    debug!("Built start args: {}", start_args);

    if !args.pretend {
        let scheduler = WindowsTaskScheduler::new()?;
        scheduler.connect()?;

        let folder = scheduler.folder("\\")?;
        folder.delete_task(TASK_NAME)?;
        let task = scheduler.new_task()?;

        let task_principal = task.principal()?;
        task_principal.set_group_id("NT AUTHORITY\\SYSTEM")?;
        task_principal.set_logon_type(TaskLogonType::Group)?;
        task_principal.set_runlevel(TaskRunlevel::Highest)?;

        let task_reginfo = task.registration_info()?;
        task_reginfo.set_author("Mathieu \"OtaK_\" Amiot")?;
        task_reginfo.set_description("Start TimerSet at logon of any user with admin permissions")?;

        let task_settings = task.settings()?;
        task_settings.set_start_when_available(true)?;
        task_settings.set_enabled(true)?;
        task_settings.set_hidden(false)?;
        task_settings.set_multiple_instances(TaskInstancesPolicy::StopExisting)?;
        task_settings.set_execution_time_limit("PT0S")?;
        task_settings.set_compatibility(TaskCompatibility::V24)?;

        let triggers = task.triggers()?;
        let raw_trigger = triggers.create(TaskTriggerType::Logon)?;
        use crate::task_scheduler::SubTrigger as _;
        let logon_trigger = LogonTrigger::new(raw_trigger)?;
        logon_trigger.trigger().set_execution_time_limit("PT0S")?;
        logon_trigger.set_delay("PT10S")?;

        let action_collection = task.actions()?;
        let raw_task = action_collection.create(TaskActionType::Exec)?;
        use crate::task_scheduler::SubAction as _;
        let exec_action = ExecAction::new(raw_task)?;
        exec_action.set_path(start_args.target)?;
        exec_action.set_arguments(StartArgs::args_to_string(&start_args.args))?;
        if let Some(start_location) = start_args.start_location {
            exec_action.set_working_directory(start_location)?;
        }

        let _ = folder.register_task_definition(
            TASK_NAME,
            task,
            TASK_CREATE_OR_UPDATE as _,
            None,
            None,
            TaskLogonType::InteractiveToken,
            None,
        )?;
    }

    info!("Installation complete");

    Ok(())
}

pub fn uninstall(args: &crate::Opts) -> std::io::Result<()> {
    let scheduler = WindowsTaskScheduler::new()?;
    scheduler.connect()?;

    let folder = scheduler.folder("\\")?;
    if !args.pretend {
        folder.delete_task(TASK_NAME)?;
    }

    // Delete files
    let mut dest_path: std::path::PathBuf = std::env::var("PROGRAMFILES").unwrap().into();
    dest_path.push("TimerSet");
    debug!("Installation path to be removed: {:?}", dest_path);
    if !args.pretend {
        let _ = std::fs::remove_dir_all(dest_path);
    }

    info!("Uninstall complete.");
    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn should_format_args_correctly() {
        let test_args = super::StartArgs {
            target: "timerset.exe".into(),
            args: vec!["--arg1".into(), "--arg2".into(), "--arg3 withvalue".into()],
            start_location: None,
        };
        assert_eq!(
            &format!("{}", test_args),
            "\"timerset.exe\" --arg1 --arg2 --arg3 withvalue"
        );
    }
}
