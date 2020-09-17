# timerset-rs

TimerSet is an utility to check and modify the Windows Timer Resolution.

Since v0.3.0: Also includes a feature to periodically empty the Windows StandbyList, notorious for causing stutters when gaming on Windows 10 systems after Windows 10 version 1809.

## PSA: Please be aware that raising your Timer Resolution on a laptop has huge effects on battery life and is generally a bad idea

## Usage

The program should be run from an elevated (ie. cmd => run as administrator) terminal prompt.

Obviously, it only works on Windows. Confirmed to work on Windows 10 2004, but should run on pretty much any version of Windows since the used APIs are so old.

```text
timerset 0.3.0
Mathieu Amiot <amiot.mathieu@gmail.com>
TimerSet allows you to change your NT Kernel system timer
Also allows you to monitor Windows Standby List and clean it up when needed

USAGE:
    timerset.exe [FLAGS] [OPTIONS]

FLAGS:
        --islc         Enables Windows Standby List periodic cleaning. It is akin to how ISLC by Wagnard works. Please
                       note that when enabling this, the program will **NOT** be idle at all times and will periodically
                       poll the system memory to check whether a cleanup is needed or not
    -h, --help         Prints help information
    -i, --install      Installs TimerSet to your system and runs it on startup
    -u, --uninstall    Uninstalls TimerSet from your system
    -v, --values       Prints the possible timer value range for your system. Please note that it can depend on many
                       factors such as HPET or dynamic/synthetic timers enabled or disabled
    -V, --version      Prints version information

OPTIONS:
        --islc-timer <clean-standby-list-poll-freq>
            Standby List periodic cleaning poll interval. Defaults to 10 seconds which should be enough for most systems
            without impacting performance [default: 10]
        --cscm <clear-standby-cached-mem>
            Cached memory threshold where the Windows Standby List will be cleared (in MB) Defaults to 1024MB (1GB)
            [default: 1024]
        --csfm <clear-standby-free-mem>
            Free memory threshold where the Windows Standby List will be cleared (in MB) Defaults to 1024MB (1GB)
            [default: 1024]
    -t, --timer <timer>
            Allows to set a custom timer value in μs. Will be clamped between the bounds of allowed timer values. Also
            note that sometimes, setting high timer values are rejected by the system and will be lowered down depending
            on which clock source your system is using (TSC tends to lower values by 5μs, HPET does not for instance)
```

## Examples

### Installing in your system with lowest possible timer values

`timerset.exe --install`

### Uninstall the program from your system

`timerset.exe --uninstall`

### Install the program with a custom timer (here, 2ms)

`timerset.exe --install --timer 2000`

### Install the program with an automatic lowest-possible timer and standby-list cleaning capabilities

`timerset.exe --install --islc`

### Display the timer range on your system

`timerset.exe --values`

## Building & Contributing

Requirements:

- Rust
- Windows 10 SDK

`cargo build --release` and you should be good!

## Authors

Mathieu "@OtaK_" Amiot
