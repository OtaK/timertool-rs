fn main() {
    if cfg!(not(windows)) {
        panic!("No idea how you compiled this but this software is only compatible with Windows.");
    }
}
