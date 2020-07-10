use log::info;

fn main() {
    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();

    let mut min_res = 0u32;
    let mut max_res = 0u32;
    let mut cur_res = 0u32;
    unsafe {
        winapi::um::nttimer::NtQueryTimerResolution(&mut min_res, &mut max_res, &mut cur_res);
    }

    info!("timers: min [{}] / max [{}] / cur [{}]", min_res, max_res, cur_res);

    unsafe {
        winapi::um::nttimer::NtSetTimerResolution(max_res, 1, &mut cur_res);
    }

    info!("cur timer after trying max resolution [{}]", cur_res);
}
