#[cfg(not(windows))]
fn main() {
    panic!("This software is only compatible with Windows.");
}


#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_manifest_file("./assets/timerset.manifest");
    res.set_icon("./assets/timerset-icon.ico");

    res.compile().unwrap();
}
