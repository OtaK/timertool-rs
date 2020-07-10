# timerset-rs

TimerSet is an utility to check and modify the Windows Timer Resolution.

## PSA: Please be aware that raising your Timer Resolution on a laptop has huge effects on battery life and is generally a bad idea!

## Usage

The program should be run from an elevated (ie. cmd => run as administrator) terminal prompt.

Obviously, it only works on Windows. Confirmed to work on Windows 10 2004, but should run on pretty much any version of Windows since the used APIs are so old.

```text
timerset 0.2.0
Mathieu Amiot
TimerSet allows you to change your NT Kernel system timer

USAGE:
    timerset.exe [FLAGS] [OPTIONS]

FLAGS:
    -h, --help         Prints help information
    -i, --install      Installs TimerSet to your system and runs it on startup
    -u, --uninstall    Uninstalls TimerSet from your system
    -v, --values       Prints the possible timer value range for your system. Please note that it can depend on many
                       factors such as HPET or dynamic/synthetic timers enabled or disabled
    -V, --version      Prints version information

OPTIONS:
    -t, --timer <timer>    Allows to set a custom timer value in μs. Will be clamped between the bounds of allowed timer values.
                           Also note that sometimes, setting high timer values are rejected by the system and will be
                           lowered down according to unknown factors (windows ancient magic????)

```

## Examples

### Installing in your system with lowest possible timer values

`timerset.exe --install`

### Uninstall the program from your system

`timerset.exe --uninstall`

### Install the program with a custom timer (here, 2ms)

`timerset.exe --install --timer 2000`

### Display the timer range on your system

`timerset.exe --values`

## Building & Contributing

Requirements:

- Rust

`cargo build --release` and you should be good!

## Authors

Mathieu "@OtaK_" Amiot
