# Rust Windows Kernel Driver for Red Teamers

## Features (IN PROGRESS)

* Protect / unprotect process (Done)
* Elevate / remove token privileges (Done)
* Enumerate / remove kernel callbacks (Todo)
        - PsSetCreateProcessNotifyRoutine
        - PsSetCreateThreadNotifyRoutine
        - PsSetLoadImageNotifyRoutine
        - CmRegisterCallbackEx
        - ObRegisterCallbacks
* DSE enable/disable (Todo)
* Hide process (Todo)
* Kernel mode manual mapper (Todo)

## [Install Rust](https://www.rust-lang.org/tools/install)

"To start using Rust, download the installer, then run the program and follow the onscreen instructions. You may need to install the [Visual Studio C++ Build tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) when prompted to do so. If you are not on Windows see ["Other Installation Methods"](https://forge.rust-lang.org/infra/other-installation-methods.html).


## [Install](https://rust-lang.github.io/rustup/concepts/channels.html)

Install and change to Rust nightly

```
rustup toolchain install nightly
rustup default nightly
```

## [Install cargo-make](https://github.com/sagiegurari/cargo-make)

Install cargo-make

```
cargo install cargo-make
```

## [Install WDK/SDK](https://docs.microsoft.com/en-us/windows-hardware/drivers/download-the-wdk)

1. Step 1: Install Visual Studio 2019
2. Step 2: Install Windows 11 SDK (22000.1)
3. Step 3: Install Windows 11 WDK

## Build Driver

Change directory to .\driver\ and build driver

```
cargo make sign
```

## Build Client

```
cargo build -p client
```

## PatchGuard (Kernel Patch Protection)

PatchGuard protects the Windows Kernel from 64-bit Windows versions of Vista onwards to blue-screen if unauthorized modifications of kernel code are detected. This also prevents things like SSDT hooking which is equivalent to IAT hooking in user-mode. One of the flaws with PatchGuard is that is not constantly checking protected regions which introduces race condition flaws that allow us to modify a protected region and change it back without PatchGuard flagging it as an "unauthorized modifications". Although we won't know when PatchGuard will perform its next check and good OPSEC would be to modify the protected region for a very short amount of time and change it back so PatchGuard does not notice. Also when Windows is put into test signing / debug mode PatchGuard is disabled.

## Driver Signatures Enforcement

Since Windows 10 1607, Microsoft will not load kernel drivers unless they are signed via the Microsoft Development Portal. But if for developers this would mean getting an Extended Validation (EV) code signing certificate to sign your kernel driver that is handed out from providers such as DigiCert, and GlobalSign. Then you must join the Windows Hardware Developer Center program by submitting your Extended Validation (EV) code signing certificates and going through a vetting process. When they are accepted a driver needs to be signed by the developer with their EV cert and uploaded to the Microsoft Development Portal to be approved and signed by Microsoft. This is the "normal way" to load your driver.

Currently, this driver does not support manual mapping. However, an alternative way to load your driver is to manually map it by exploiting an existing CVE in a signed driver such as Capcom or Intel:

* https://github.com/TheCruZ/kdmapper
* https://github.com/not-wlan/drvmap
* https://github.com/zorftw/kdmapper-rs

Otherwise you can always get an [extended validation (EV) code signing certificate](https://docs.microsoft.com/en-us/windows-hardware/drivers/dashboard/get-a-code-signing-certificate) by Microsoft which goes through a "vetting" process or use a 0-day which is really up to you lol.


## Kernel Callbacks

Kernel Callbacks are used to notify a Windows Kernel Driver when a specific event occurs such as when a process is created or exits aka `PsSetCreateProcessNotifyRoutine` or when a thread is created or deleted aka `PsSetCreateThreadNotifyRoutine` or when a DLL is mapped into memory `PsSetLoadImageNotifyRoutine` or `CmRegisterCallbackEx` when a registry is created or `ObRegisterCallbacks` when a handle is created. Anti-cheats have been using these for a very long time and AVs, EDRs and Sysmon are also using these.


## Enable `Test Mode` or `Test Signing` Mode 

```
bcdedit /set testsigning on
```

### [Optional] Debug via Windbg

```
bcdedit /debug on
bcdedit /dbgsettings net hostip:<IP> port:<PORT>
```

## Create / Start Service

You can use [Service Control Manager](https://docs.microsoft.com/en-us/windows/win32/services/service-control-manager) or [OSR Driver Loader](https://www.osronline.com/article.cfm%5Earticle=157.htm) to load your driver.

```
PS C:\Users\User> sc.exe create Eagle type= kernel binPath= C:\Windows\System32\Eagle.sys
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

A better way to code Windows Kernel Drivers in Rust is to create bindings as shown in the references below. However, using someone else's bindings hides the functionality and this is why I made it the classic way unless, of course, you create your own bindings. I plan on refactoring the code in the future but for now, it will be a bit messy and incomplete.

I made this project for fun and because I really like Rust and Windows Internals. This is obviously not perfect or finished yet. if you would like to learn more about Windows Kernel Programming then feel free to check out the references below.


## References and Credits

* https://not-matthias.github.io/kernel-driver-with-rust/
* https://github.com/not-matthias/kernel-driver-with-rust/
* https://codentium.com/guides/windows-dev/
* https://github.com/StephanvanSchaik/windows-kernel-rs/
* https://github.com/rmccrystal/kernel-rs
* https://github.com/pravic/winapi-kmd-rs
* https://courses.zeropointsecurity.co.uk/courses/offensive-driver-development (Big thanks to Rastamouse)
* https://leanpub.com/windowskernelprogramming
* https://guidedhacking.com/
* https://www.unknowncheats.me/
* https://gamehacking.academy/
* https://www.vergiliusproject.com/kernels/x64
* https://www.crowdstrike.com/blog/evolution-protected-processes-part-1-pass-hash-mitigations-windows-81/
* https://discord.com/invite/rust-lang-community (Thanks to: WithinRafael, Nick12, Zuix, kpreid and many others)
* https://www.rust-lang.org/
* https://doc.rust-lang.org/book/