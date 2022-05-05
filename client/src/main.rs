use sysinfo::{Pid, SystemExt, ProcessExt};
use winapi::um::{fileapi::{CreateFileA, OPEN_EXISTING}, winnt::{GENERIC_READ, GENERIC_WRITE, FILE_SHARE_READ, FILE_SHARE_WRITE}};
use std::{ffi::CString, ptr::null_mut};
use clap::Parser;
mod kernel_interface;

/// Process Manipulation Tool
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Target process to change protection
    #[clap(short, long)]
    target: String,
    
    /// Enabled or Disable process protection
    #[clap(short, long)]
    protection: Option<String>,

    /// Enumerate kernel callbacks
    #[clap(short, long)]
    enumerate: Option<String>,
}

fn main() {

    let args = Args::parse();
    let process_id = get_process_id_by_name(args.target.as_str()) as u32;
    let protect = args.protection.unwrap();
    let enumerate = args.enumerate.unwrap();


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

    if protect.to_uppercase() == "ENABLE" {
        kernel_interface::protect_process(process_id, driver_handle);
        kernel_interface::enable_tokens(process_id, driver_handle);
    } else if protect.to_uppercase() == "DISABLE" {
        kernel_interface::unprotect_process(process_id, driver_handle);
    } else if enumerate.to_uppercase() == "TRUE" {
        kernel_interface::enumerate_callbacks(driver_handle);
    }
    else {
        panic!("[-] Invalid CLI options, use help menu");
    }

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