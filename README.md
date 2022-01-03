# timerset-rs

TimerSet is an utility to check and modify the Windows Timer Resolution.

Since v0.3.0: Also includes a feature to periodically empty the Windows StandbyList, notorious for causing stutters when gaming on Windows 10 systems after Windows 10 version 1809.

## PSA: Please be aware that raising your Timer Resolution on a laptop has huge effects on battery life and is generally a bad idea

## Usage

The program should be run from an elevated (ie. cmd => run as administrator) terminal prompt.

Obviously, it only works on Windows. Confirmed to work on Windows 10 2004, but should run on pretty much any version of Windows since the used APIs are so old.

```text
timerset 0.4.1
Mathieu Amiot <amiot.mathieu@gmail.com>
TimerSet allows you to change your NT Kernel system timer Also allows you to monitor Windows Standby
List and clean it up when needed

USAGE:
    timerset.exe [OPTIONS]

OPTIONS:
        --cscm <CLEAR_STANDBY_CACHED_MEM>
            Cached memory threshold where the Windows Standby List will be cleared (in MB) Defaults
            to 1024MB (1GB)

            [default: 1024]

        --csfm <CLEAR_STANDBY_FREE_MEM>
            Free memory threshold where the Windows Standby List will be cleared (in MB) Defaults to
            1024MB (1GB)

            [default: 1024]

    -h, --help
            Print help information

    -i, --install
            Installs TimerSet to your system and runs it on startup

        --islc
            Enables Windows Standby List periodic cleaning. It is akin to how ISLC by Wagnard works

        --islc-timer <CLEAN_STANDBY_LIST_POLL_FREQ>
            Standby List anti-kernel DOS throttle timer It exists because
            CreateMemoryResourceNotification can trigger LowMemoryResourceNotifications thousands of
            times per second when they happen (i.e. every system page allocation in a high memory
            pressure situation, often 4KB) resulting in the memory list cleaning paralyzing the
            system with thousands of tries per second

            Defaults to 10 seconds which should be enough for most systems without impacting
            performance.

            [default: 10]

    -p, --pretend
            Shows the actions taken but do not modify anything on the system; Also known as a dry
            run

    -t, --timer <TIMER>
            Allows to set a custom timer value in μs. Will be clamped between the bounds of allowed
            timer values. Also note that sometimes, setting high timer values are rejected by the
            system and will be lowered down depending on which clock source your system is using
            (TSC tends to lower values by ~5μs, HPET does not for instance)

    -u, --uninstall
            Uninstalls TimerSet from your system

    -v, --values
            Prints the possible timer value range for your system. Please note that it can depend on
            many factors such as HPET or dynamic/synthetic timers enabled or disabled

    -V, --version
            Print version information

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
