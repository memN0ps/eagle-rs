use std::{mem::size_of, ptr::null_mut};
use winapi::{um::{ioapiset::DeviceIoControl}, ctypes::c_void};
use common::{TargetProcess, IOCTL_PROCESS_PROTECT_REQUEST, IOCTL_PROCESS_UNPROTECT_REQUEST};

/// Protect a process as PsProtectedSignerWinTcb
pub fn protect_process(process_id: u32, driver_handle: *mut c_void) {
    let bytes: u32 = 0;
    
    let mut target_process = TargetProcess {
        process_id: process_id,
    };
    
    let device_io_control_result = unsafe { 
        DeviceIoControl(driver_handle,
        IOCTL_PROCESS_PROTECT_REQUEST,
        &mut target_process as *mut _ as *mut c_void,
        size_of::<TargetProcess> as u32,
        null_mut(),
        0,
        bytes as *mut u32,
        null_mut())
    };

    if device_io_control_result == 0 {
        panic!("[-] Failed to call DeviceIoControl");
    }
}

/// Remove the protection of a process
pub fn unprotect_process(process_id: u32, driver_handle: *mut c_void) {
    let bytes: u32 = 0;
    
    let mut target_process = TargetProcess {
        process_id: process_id,
    };
    
    let device_io_control_result = unsafe { 
        DeviceIoControl(driver_handle,
        IOCTL_PROCESS_UNPROTECT_REQUEST,
        &mut target_process as *mut _ as *mut c_void,
        size_of::<TargetProcess> as u32,
        null_mut(),
        0,
        bytes as *mut u32,
        null_mut())
    };

    if device_io_control_result == 0 {
        panic!("[-] Failed to call DeviceIoControl");
    }
}