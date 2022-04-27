# Rust Windows Kernel Driver

## Features

* Unprotect a Process via Windows Kernel
* Protect a Process via Windows Kernel
* Read Virtual Memory via Windows Kernel (Todo)
* Write Virtual Memory via Windows Kernel (Todo)
* Adjust Process Protection (ToDo)
* Remove Kernel Callbacks (Todo)

## Install

```
cargo install --force cargo-make
```

## Build Driver

```
cargo make sign
```

## Build Client

```
cargo build -p fire-rs
```

## Loading

### Debug via Windbg

```
bcdedit /debug on
bcdedit /set testsigning on
bcdedit /dbgsettings net hostip:<IP> port:<PORT>
```

```
PS C:\Users\User> sc.exe create Eagle type= kernel binPath= C:\Eagle.sys
[SC] CreateService SUCCESS
PS C:\Users\User> sc.exe query Eagle

SERVICE_NAME: Eagle
        TYPE               : 1  KERNEL_DRIVER
        STATE              : 1  STOPPED
        WIN32_EXIT_CODE    : 1077  (0x435)
        SERVICE_EXIT_CODE  : 0  (0x0)
        CHECKPOINT         : 0x0
        WAIT_HINT          : 0x0
PS C:\Users\User> sc.exe start Eagle

SERVICE_NAME: Eagle
        TYPE               : 1  KERNEL_DRIVER
        STATE              : 4  RUNNING
                                (STOPPABLE, NOT_PAUSABLE, IGNORES_SHUTDOWN)
        WIN32_EXIT_CODE    : 0  (0x0)
        SERVICE_EXIT_CODE  : 0  (0x0)
        CHECKPOINT         : 0x0
        WAIT_HINT          : 0x0
        PID                : 0
        FLAGS              :
PS C:\Users\User> sc.exe stop Eagle

SERVICE_NAME: Eagle
        TYPE               : 1  KERNEL_DRIVER
        STATE              : 1  STOPPED
        WIN32_EXIT_CODE    : 0  (0x0)
        SERVICE_EXIT_CODE  : 0  (0x0)
        CHECKPOINT         : 0x0
        WAIT_HINT          : 0x0
```


## Note

A better way to code Windows Kernel Drivers in Rust is to create bindings as shown in the references below. However, using someone else's bindings hides the functionality and this is why I made it the classic way unless of course you create your own bindings.

I made this project for fun and because I really like Rust and Windows Internals. This is obviously not perfect or finished yet. if you would like to learn more about Windows Kernel Programming then feel free to check out the references below.


## References and Credits

* https://not-matthias.github.io/kernel-driver-with-rust/
* https://github.com/not-matthias/kernel-driver-with-rust/
* https://codentium.com/guides/windows-dev/
* https://github.com/StephanvanSchaik/windows-kernel-rs/
* https://github.com/rmccrystal/kernel-rs
* https://github.com/pravic/winapi-kmd-rs
* https://courses.zeropointsecurity.co.uk/courses/offensive-driver-development
* https://leanpub.com/windowskernelprogramming
* https://guidedhacking.com/
* https://www.unknowncheats.me/
* https://gamehacking.academy/