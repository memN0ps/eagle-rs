#![no_std]

mod string;
mod process;
mod token;
mod callbacks;

use core::mem::size_of;
use core::panic::PanicInfo;
use core::ptr::null_mut;
use kernel_print::kernel_println;
use winapi::km::wdm::IO_PRIORITY::IO_NO_INCREMENT;
use winapi::km::wdm::{DRIVER_OBJECT, IoCreateDevice, PDEVICE_OBJECT, IoCreateSymbolicLink, IRP_MJ, DEVICE_OBJECT, IRP, IoCompleteRequest, IoGetCurrentIrpStackLocation, IoDeleteSymbolicLink, IoDeleteDevice, DEVICE_TYPE};
use winapi::shared::ntdef::{NTSTATUS, UNICODE_STRING, FALSE, NT_SUCCESS, TRUE};
use winapi::shared::ntstatus::{STATUS_SUCCESS, STATUS_UNSUCCESSFUL};
use common::{TargetProcess, IOCTL_PROCESS_READ_REQUEST, IOCTL_PROCESS_WRITE_REQUEST, IOCTL_PROCESS_PROTECT_REQUEST, IOCTL_PROCESS_UNPROTECT_REQUEST, IOCTL_PROCESS_TOKEN_PRIVILEGES_REQUEST, IOCTL_PROCESS_NOTIFY_CALLBACK};
use crate::callbacks::{PsSetCreateProcessNotifyRoutineEx, process_create_callback, PcreateProcessNotifyRoutineEx};
use crate::process::{protect_process, unprotect_process};
use crate::string::create_unicode_string;
use crate::token::enable_all_token_privileges;
extern crate alloc;

/// When using the alloc crate it seems like it does some unwinding. Adding this
/// export satisfies the compiler but may introduce undefined behaviour when a
/// panic occurs.
#[no_mangle]
pub extern "system" fn __CxxFrameHandler3(_: *mut u8, _: *mut u8, _: *mut u8, _: *mut u8) -> i32 { unimplemented!() }

#[global_allocator]
static GLOBAL: kernel_alloc::KernelAlloc = kernel_alloc::KernelAlloc;

/// Explanation can be found here: https://github.com/Trantect/win_driver_example/issues/4
#[export_name = "_fltused"]
static _FLTUSED: i32 = 0;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "system" fn driver_entry(driver: &mut DRIVER_OBJECT, _: &UNICODE_STRING) -> NTSTATUS {
    //unsafe { DbgPrint("Hello, world!\0".as_ptr()); }
    kernel_println!("[+] Driver Entry called");

    driver.DriverUnload = Some(driver_unload);

    driver.MajorFunction[IRP_MJ::CREATE as usize] = Some(dispatch_create_close);
    driver.MajorFunction[IRP_MJ::CLOSE as usize] = Some(dispatch_create_close);
    driver.MajorFunction[IRP_MJ::DEVICE_CONTROL as usize] = Some(dispatch_device_control);

    let device_name = create_unicode_string(obfstr::wide!("\\Device\\Eagle\0"));
    let mut device_object: PDEVICE_OBJECT = null_mut();
    let mut status = unsafe { 
        IoCreateDevice(
            driver,
            0,
            &device_name,
            DEVICE_TYPE::FILE_DEVICE_UNKNOWN,
            0,
            FALSE, 
            &mut device_object
        ) 
    };

    if !NT_SUCCESS(status) {
        kernel_println!("[-] Failed to create device object ({:#x})", status);
        return status;
    }

    let symbolic_link = create_unicode_string(obfstr::wide!("\\??\\Eagle\0"));
    status = unsafe { IoCreateSymbolicLink(&symbolic_link, &device_name) };

    if !NT_SUCCESS(status) {
        kernel_println!("[-] Failed to create symbolic link ({:#x})", status);
        return status;
    }

    //ProcessNotify (called when a process is created)
    unsafe { PsSetCreateProcessNotifyRoutineEx(process_create_callback as PcreateProcessNotifyRoutineEx, FALSE) };


    return STATUS_SUCCESS;
}


pub extern "system" fn dispatch_device_control(_device_object: &mut DEVICE_OBJECT, irp: &mut IRP) -> NTSTATUS {

    kernel_println!("[+] IRP_MJ_DEVICE_CONTROL called");
    
    let stack = IoGetCurrentIrpStackLocation(irp);
    let control_code = unsafe { (*stack).Parameters.DeviceIoControl().IoControlCode };
    let mut status = STATUS_UNSUCCESSFUL;
    let mut byte_io: usize = 0;

    match control_code {
        IOCTL_PROCESS_READ_REQUEST => {
            kernel_println!("[*] IOCTL_PROCESS_READ_REQUEST");
            //kernel_read_virtual_memory();
        },
        IOCTL_PROCESS_WRITE_REQUEST => {
            kernel_println!("[*] IOCTL_PROCESS_WRITE_REQUEST");
            //kernel_write_virtual_memory()
        },
        IOCTL_PROCESS_PROTECT_REQUEST => {
            kernel_println!("[*] IOCTL_PROCESS_PROTECT_REQUEST");
            let protect_process_status = protect_process(irp, stack);
           
            if NT_SUCCESS(protect_process_status) {
                kernel_println!("[+] Process protection successful");
                byte_io = size_of::<TargetProcess>();
                status = STATUS_SUCCESS;
            } else {
                kernel_println!("[-] Process protection failed");
            }
        },
        IOCTL_PROCESS_UNPROTECT_REQUEST => {
            kernel_println!("[*] IOCTL_PROCESS_UNPROTECT_REQUEST");
            let unprotect_process_status = unprotect_process(irp, stack);
           
            if NT_SUCCESS(unprotect_process_status) {
                kernel_println!("[+] Process unprotection successful");
                byte_io = size_of::<TargetProcess>();
                status = STATUS_SUCCESS;
            } else {
                kernel_println!("[-] Process unprotection failed");
            }
        },
        IOCTL_PROCESS_TOKEN_PRIVILEGES_REQUEST => {
            kernel_println!("[*] IOCTL_PROCESS_TOKEN_PRIVILEGES_REQUEST");
            let token_privs_status = enable_all_token_privileges(irp, stack);
           
            if NT_SUCCESS(token_privs_status) {
                kernel_println!("[+] Process token privileges successful");
                byte_io = size_of::<TargetProcess>();
                status = STATUS_SUCCESS;
            } else {
                kernel_println!("[-] Process token privileges failed");
            }
        },
        _ => {
            kernel_println!("[-] Invalid IOCTL code")
        },
    }

    unsafe { *(irp.IoStatus.__bindgen_anon_1.Status_mut()) = status };
    irp.IoStatus.Information = byte_io;
    unsafe { IoCompleteRequest(irp, IO_NO_INCREMENT) };

    return STATUS_SUCCESS;
}

pub extern "system" fn dispatch_create_close(_device_object: &mut DEVICE_OBJECT, irp: &mut IRP) -> NTSTATUS {
    let stack = IoGetCurrentIrpStackLocation(irp);
    let code = unsafe { (*stack).MajorFunction };

	if code == IRP_MJ::CREATE as u8 {
		kernel_println!("[+] IRP_MJ_CREATE called");
	} else {
		kernel_println!("[+] IRP_MJ_CLOSE called");
	}
	
    irp.IoStatus.Information = 0;
    unsafe { *(irp.IoStatus.__bindgen_anon_1.Status_mut()) = STATUS_SUCCESS };

    unsafe { IoCompleteRequest(irp, IO_NO_INCREMENT) };
    
    return STATUS_SUCCESS;
}

pub extern "system" fn driver_unload(driver: &mut DRIVER_OBJECT) {
    let symbolic_link = create_unicode_string(obfstr::wide!("\\??\\Eagle\0"));
    unsafe { IoDeleteSymbolicLink(&symbolic_link) };
    unsafe { IoDeleteDevice(driver.DeviceObject) };

    // Remove Callbacks (or BSOD)
    unsafe { PsSetCreateProcessNotifyRoutineEx(process_create_callback as PcreateProcessNotifyRoutineEx, TRUE) };
    kernel_println!("[+] Driver unloaded successfully!");
}