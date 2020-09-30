use log::{debug, info};

use winapi::{
    shared::{
        ntdef::NULL,
        rpcdce::{RPC_C_AUTHN_LEVEL_PKT_PRIVACY, RPC_C_IMP_LEVEL_IMPERSONATE},
        wtypes::{VARIANT_TRUE, VARIANT_FALSE},
        wtypesbase::CLSCTX_INPROC_SERVER,
    },
    um::{
        combaseapi::{CoCreateInstance, CoInitializeEx, CoInitializeSecurity, CoUninitialize},
        oaidl::{VARIANT, VARIANTARG},
        objbase::COINIT_APARTMENTTHREADED,
        oleauto::VariantInit,
        taskschd::{
            IAction, IActionCollection, IExecAction, ILogonTrigger, IPrincipal, IRegisteredTask,
            IRegistrationInfo, ITaskDefinition, ITaskFolder, ITaskService, ITaskSettings, ITrigger,
            ITriggerCollection, TaskScheduler, TASK_ACTION_EXEC, TASK_CREATE_OR_UPDATE,
            TASK_LOGON_GROUP, TASK_RUNLEVEL_HIGHEST, TASK_TRIGGER_LOGON, TASK_LOGON_INTERACTIVE_TOKEN,
            TASK_COMPATIBILITY_V2_4,
        },
    },
};

#[allow(dead_code)]
const TASK_NAME: &str = "Start TimerSet [DEV]";


#[inline(always)]
fn empty_variant() -> VARIANT {
    let mut variant: VARIANTARG = unsafe { std::mem::zeroed() };
    unsafe { VariantInit(&mut variant as *mut _ as _) };
    debug!("Initialized empty variant with type {:?}", unsafe {
        variant.n1.n2().vt
    });
    variant
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct StartArgs {
    target: String,
    args: Vec<String>,
    start_location: Option<String>
}

impl StartArgs {
    pub fn args_to_string(args: &Vec<String>) -> String {
        args.iter()
            .enumerate()
            .fold(String::new(), |mut s, (i, arg)| {
                if i > 0 {
                    s.push(' ');
                }
                s.push_str(&arg);
                s
            })
    }

    fn build_from_args(mut dest_path: std::path::PathBuf, args: &crate::Opts) -> Self {
        let mut ret = Self::default();
        ret.target = format!("{}", dest_path.display());
        if dest_path.pop() {
            ret.start_location = dest_path.to_str().map(Into::into);
        }

        if let Some(timer_value) = args.timer {
            ret.args.push(format!("--timer {}", timer_value));
        }

        if args.clean_standby_list {
            ret.args.push("--islc".to_string());
            if args.clean_standby_list_poll_freq != 10 {
                ret.args.push(format!(
                    "--islc-timer {}",
                    args.clean_standby_list_poll_freq
                ));
            }
            if args.clear_standby_cached_mem != 1024 {
                ret.args.push(format!("--cscm {}", args.clear_standby_cached_mem));
            }
            if args.clear_standby_free_mem != 1024 {
                ret.args.push(format!("--csfm {}", args.clear_standby_free_mem));
            }
        }

        ret
    }
}

impl std::fmt::Display for StartArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\" {}", self.target, Self::args_to_string(&self.args))
    }
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

    let start_args = StartArgs::build_from_args(
        dest_path,
        args,
    );

    debug!("Built start args: {}", start_args);

    // let mut scheduler = crate::task_scheduler::WindowsTaskScheduler::new()?;
    // scheduler.connect()?;
    // scheduler.fetch_folder("\\")?;

    crate::w32_ok!(DEBUG CoInitializeEx(
        NULL,
        COINIT_APARTMENTTHREADED
    ), |result| debug!(
        "CoInitializeEx(NULL, COINIT_APARTMENTTHREADED) -> {}", result
    ))?;

    crate::w32_ok!(CoInitializeSecurity(
        NULL,
        -1,
        NULL as _,
        NULL,
        RPC_C_AUTHN_LEVEL_PKT_PRIVACY,
        RPC_C_IMP_LEVEL_IMPERSONATE,
        NULL,
        0,
        NULL,
    ), ELSE CoUninitialize())?;

    debug!("CoInitializeSecurity()");

    use winapi::{Class as _, Interface as _};
    let iid_itaskservice = ITaskService::uuidof();
    let clsid = TaskScheduler::uuidof();
    let mut task_service: *mut ITaskService = unsafe { std::mem::zeroed() };
    crate::w32_ok!(DEBUG CoCreateInstance(
        &clsid as _,
        NULL as _,
        CLSCTX_INPROC_SERVER,
        &iid_itaskservice as _,
        &mut task_service as *mut *mut _ as _,
    ), |result| debug!(
        "CoCreateInstance({:?}, NULL, CLSCTX_INPROC_SERVER, {:?}, &task_service) -> {}",
        clsid,
        iid_itaskservice,
        result
    ))?;

    crate::w32_ok!(DEBUG (*task_service).Connect(
        empty_variant(),
        empty_variant(),
        empty_variant(),
        empty_variant(),
    ), |result| {
        debug!("ITaskService::Connect() -> {:#X}", result);
    }, ELSE {
        (*task_service).Release();
        CoUninitialize();
    })?;

    debug!("ITaskService::Connect() successful");

    let mut task_folder: *mut ITaskFolder = unsafe { std::mem::zeroed() };

    let task_folder_name = crate::wstr!("\\");

    crate::w32_ok!(
        (*task_service).GetFolder(task_folder_name, &mut task_folder as *mut *mut _ as _),
        ELSE {
            (*task_service).Release();
            CoUninitialize();
        }
    )?;

    debug!("ITaskService::GetFolder() successful");

    let task_name = crate::wstr!(TASK_NAME);
    let _ = unsafe { (*task_folder).DeleteTask(task_name, 0) };
    debug!("ITaskFolder::DeleteTask() successful");

    let mut task_definition: *mut ITaskDefinition = unsafe { std::mem::zeroed() };
    crate::w32_ok!(
        (*task_service).NewTask(0, &mut task_definition as *mut *mut _ as _),
        ELSE {
            (*task_service).Release();
            (*task_folder).Release();
            CoUninitialize();
        }
    )?;

    debug!(
        "ITaskService::NewTask() successful; ITaskDefinition: {:?}",
        task_definition
    );

    unsafe { (*task_service).Release() };

    let mut task_principal: *mut IPrincipal = unsafe { std::mem::zeroed() };
    crate::w32_ok!((*task_definition).get_Principal(&mut task_principal as *mut *mut _))?;
    debug!(
        "ITaskDefinition::get_Principal() successful; IPrincipal: {:?}",
        task_principal
    );
    crate::w32_ok!((*task_principal).put_GroupId(crate::wstr!("NT AUTHORITY\\SYSTEM")), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;
    crate::w32_ok!((*task_principal).put_LogonType(TASK_LOGON_GROUP), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;
    crate::w32_ok!((*task_principal).put_RunLevel(TASK_RUNLEVEL_HIGHEST), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;

    let mut task_registration_info: *mut IRegistrationInfo = std::ptr::null_mut();
    crate::w32_ok!((*task_definition).get_RegistrationInfo(&mut task_registration_info as *mut *mut _), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;
    debug!("ITaskDefinition::get_RegistrationInfo() successful");
    crate::w32_ok!((*task_registration_info).put_Author(crate::wstr!("Mathieu \"OtaK_\" Amiot")), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;
    debug!("IRegistrationInfo::put_Author(\"Mathieu \\\"OtaK_\\\" Amiot\") successful");
    crate::w32_ok!((*task_registration_info).put_Description(crate::wstr!("Start TimerSet at logon of any user with admin permissions")), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;
    debug!("IRegistrationInfo::put_Description(\"Start TimerSet at logon of any user with admin permissions\") successful");
    unsafe { (*task_registration_info).Release() };

    let mut task_settings: *mut ITaskSettings = std::ptr::null_mut();
    crate::w32_ok!((*task_definition).get_Settings(&mut task_settings as *mut *mut _ as _), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;
    debug!(
        "ITaskDefinition::get_Settings() successful; ITaskSettings: {:?}",
        task_settings
    );

    crate::w32_ok!((*task_settings).put_StartWhenAvailable(VARIANT_TRUE), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;
    debug!("ITaskSettings::put_StartWhenAvailable(TRUE) success");
    crate::w32_ok!((*task_settings).put_Enabled(VARIANT_TRUE), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;
    debug!("ITaskSettings::put_Enabled(TRUE) success");
    crate::w32_ok!((*task_settings).put_Hidden(VARIANT_FALSE), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;
    debug!("ITaskSettings::put_Hidden(FALSE) success");
    crate::w32_ok!((*task_settings).put_MultipleInstances(winapi::um::taskschd::TASK_INSTANCES_STOP_EXISTING), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;
    debug!("ITaskSettings::put_MultipleInstances(TASK_INSTANCES_STOP_EXISTING) success");
    crate::w32_ok!((*task_settings).put_ExecutionTimeLimit(crate::wstr!("PT0S")), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;
    debug!("ITaskSettings::put_ExecutionTimeLimit(\"PT0S\\0\") success");

    crate::w32_ok!((*task_settings).put_Compatibility(TASK_COMPATIBILITY_V2_4), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;
    debug!("ITaskSettings::put_ExecutionTimeLimit(\"PT0S\\0\") success");

    unsafe { (*task_settings).Release() };

    let mut trigger_collection: *mut ITriggerCollection = std::ptr::null_mut();
    crate::w32_ok!((*task_definition).get_Triggers(&mut trigger_collection as *mut *mut _ as _), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;
    debug!(
        "ITaskDefinition::get_Triggers() successful; ITriggerCollection: {:?}",
        trigger_collection
    );

    let mut trigger: *mut ITrigger = std::ptr::null_mut();
    crate::w32_ok!((*trigger_collection).Create(TASK_TRIGGER_LOGON, &mut trigger as *mut *mut _ as _), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;
    debug!(
        "ITriggerCollection::Create() successful; ITrigger: {:?}",
        trigger
    );

    unsafe { (*trigger_collection).Release() };

    let mut logon_trigger: *mut ILogonTrigger = std::ptr::null_mut();
    crate::w32_ok!(
        (*trigger).QueryInterface(&ILogonTrigger::uuidof() as _, &mut logon_trigger as *mut *mut _ as _),
        ELSE {
            (*task_registration_info).Release();
            (*task_folder).Release();
            CoUninitialize();
        }
    )?;
    debug!(
        "ITrigger::QueryInterface() successful: ILogonTrigger: {:?}",
        logon_trigger
    );

    unsafe { (*trigger).Release() };
    crate::w32_ok!((*logon_trigger).put_ExecutionTimeLimit(crate::wstr!("PT0S")), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;
    debug!("ILogonTrigger::put_ExecutionTimeLimit(\"PT0S\\0\") successful");
    crate::w32_ok!((*logon_trigger).put_Delay(crate::wstr!("PT10S")), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;
    debug!("ILogonTrigger::put_Delay(\"PT10S\\0\") successful");
    unsafe { (*logon_trigger).Release() };

    let mut action_collection: *mut IActionCollection = std::ptr::null_mut();
    crate::w32_ok!((*task_definition).get_Actions(&mut action_collection as *mut _ as _), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;
    debug!(
        "ITaskDefinition::get_Actions() successful; IActionCollection: {:?}",
        action_collection
    );
    let mut action: *mut IAction = std::ptr::null_mut();
    crate::w32_ok!((*action_collection).Create(TASK_ACTION_EXEC, &mut action as *mut *mut _ as _), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;
    debug!(
        "IActionCollection::Create() successful; IAction: {:?}",
        action
    );
    unsafe { (*action_collection).Release() };
    let mut exec_action: *mut IExecAction = std::ptr::null_mut();
    crate::w32_ok!((*action).QueryInterface(&IExecAction::uuidof() as _, &mut exec_action as *mut _ as _), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;
    debug!(
        "IAction::QueryInterface() successful; IExecAction: {:?}",
        exec_action
    );
    unsafe { (*action).Release() };

    crate::w32_ok!((*exec_action).put_Path(crate::wstr!(start_args.target)), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;
    debug!("IExecAction::put_Path({}) successful", start_args.target);
    let args = StartArgs::args_to_string(&start_args.args);
    crate::w32_ok!((*exec_action).put_Arguments(crate::wstr!(args)), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;
    debug!("IExecAction::put_Arguments({}) successful", args);

    if let Some(start_location) = start_args.start_location {
        crate::w32_ok!((*exec_action).put_WorkingDirectory(crate::wstr!(start_location)), ELSE {
            (*task_folder).Release();
            (*task_definition).Release();
            CoUninitialize();
        })?;
        debug!("IExecAction::put_WorkingDirectory({}) successful", start_location);
    }

    unsafe { (*exec_action).Release() };
    let mut registered_task: *mut IRegisteredTask = NULL as _;

    let empty_string = crate::wstr!("");

    #[allow(unused_unsafe)]
    crate::w32_ok!(DEBUG (*task_folder).RegisterTaskDefinition(
        crate::wstr!(TASK_NAME),
        task_definition,
        TASK_CREATE_OR_UPDATE as _,
        empty_variant(),
        empty_variant(),
        TASK_LOGON_INTERACTIVE_TOKEN,
        crate::bstr_variant!(empty_string),
        &mut registered_task,
    ), |result| debug!("ITaskFolder::RegisterTaskDefinition() -> {:#X}", result), ELSE {
        (*task_folder).Release();
        (*task_definition).Release();
        CoUninitialize();
    })?;

    debug!(
        "ITaskFolder::RegisterTaskDefinition() successful; IRegisteredTask: {:?}",
        registered_task
    );

    unsafe {
        (*task_folder).Release();
        (*task_definition).Release();
        (*registered_task).Release();
        CoUninitialize();
    }

    info!("Installation complete.");
    Ok(())
}

pub fn uninstall() -> std::io::Result<()> {
    crate::w32_ok!(CoInitializeEx(NULL, COINIT_APARTMENTTHREADED))?;
    crate::w32_ok!(CoInitializeSecurity(
        NULL,
        -1,
        NULL as _,
        NULL,
        RPC_C_AUTHN_LEVEL_PKT_PRIVACY,
        RPC_C_IMP_LEVEL_IMPERSONATE,
        NULL,
        0,
        NULL,
    ), ELSE CoUninitialize())?;

    use winapi::{Class as _, Interface as _};
    let iid_itaskservice = ITaskService::uuidof();
    let clsid = TaskScheduler::uuidof();
    let mut task_service: *mut ITaskService = std::ptr::null_mut();
    crate::w32_ok!(CoCreateInstance(
        &clsid as _,
        NULL as _,
        CLSCTX_INPROC_SERVER,
        &iid_itaskservice as _,
        &mut task_service as *mut _ as _,
    ))?;

    crate::w32_ok!((*task_service).Connect(
        std::mem::zeroed(),
        std::mem::zeroed(),
        std::mem::zeroed(),
        std::mem::zeroed(),
    ), ELSE CoUninitialize())?;

    let mut task_folder: *mut ITaskFolder = std::ptr::null_mut();

    let task_folder_name = crate::wstr!("\\");

    crate::w32_ok!(
        (*task_service).GetFolder(task_folder_name, &mut task_folder as *mut _ as _),
        ELSE CoUninitialize()
    )?;

    unsafe {
        (*task_service).Release();
        std::ptr::drop_in_place(task_service);
    }

    let task_name = crate::wstr!(TASK_NAME);
    let _ = unsafe { (*task_folder).DeleteTask(task_name, 0) };
    unsafe {
        (*task_folder).Release();
        std::ptr::drop_in_place(task_folder);
        CoUninitialize();
    }

    // Delete files
    let mut dest_path: std::path::PathBuf = std::env::var("PROGRAMFILES").unwrap().into();
    dest_path.push("TimerSet");
    debug!("Installation path to be removed: {:?}", dest_path);
    let _ = std::fs::remove_dir_all(dest_path);

    info!("Uninstall complete.");

    Ok(())
}

// fn get_startup_key() -> std::io::Result<winreg::RegKey> {
//     use winreg::enums::*;
//     let hklm = winreg::RegKey::predef(HKEY_LOCAL_MACHINE);
//     hklm.open_subkey_with_flags(
//         "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
//         KEY_SET_VALUE,
//     )
// }

#[cfg(test)]
mod test {
    #[test]
    fn should_format_args_correctly() {
        let test_args = super::StartArgs {
            target: "timerset.exe".into(),
            args: vec![
                "--arg1".into(),
                "--arg2".into(),
                "--arg3 withvalue".into(),
            ],
            start_location: None,
        };
        assert_eq!(
            &format!("{}", test_args),
            "\"timerset.exe\" --arg1 --arg2 --arg3 withvalue"
        );
    }
}
