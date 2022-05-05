#![no_std]

mod string;
mod process;
mod token;
mod callbacks;
pub mod includes;


use core::{panic::PanicInfo, mem::size_of};
use core::ptr::null_mut;
use winapi::{km::wdm::IO_PRIORITY::IO_NO_INCREMENT, shared::ntstatus::{STATUS_INVALID_PARAMETER, STATUS_BUFFER_TOO_SMALL, STATUS_NOT_FOUND}};
use winapi::km::wdm::{DRIVER_OBJECT, IoCreateDevice, PDEVICE_OBJECT, IoCreateSymbolicLink, IRP_MJ, DEVICE_OBJECT, IRP, IoCompleteRequest, IoGetCurrentIrpStackLocation, IoDeleteSymbolicLink, IoDeleteDevice, DEVICE_TYPE};
use winapi::shared::ntdef::{NTSTATUS, UNICODE_STRING, FALSE, NT_SUCCESS, TRUE};
use winapi::shared::ntstatus::{STATUS_SUCCESS, STATUS_UNSUCCESSFUL};
use common::{IOCTL_PROCESS_PROTECT_REQUEST, IOCTL_PROCESS_UNPROTECT_REQUEST, IOCTL_PROCESS_TOKEN_PRIVILEGES_REQUEST, IOCTL_PROCESS_ENUM_CALLBACKS, CallBackInformation, TargetProcess};
use crate::{callbacks::{PsSetCreateProcessNotifyRoutineEx, process_create_callback, PcreateProcessNotifyRoutineEx, search_loaded_modules}, process::find_psp_set_create_process_notify};
use crate::process::{protect_process, unprotect_process};
use crate::string::create_unicode_string;
use crate::token::enable_all_token_privileges;
extern crate alloc;
use kernel_log::KernelLogger;
use log::{LevelFilter};


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
    KernelLogger::init(LevelFilter::Info).expect("Failed to initialize logger");

    log::info!("Driver Entry called");
    //find_psp_set_create_process_notify();

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
        log::error!("Failed to create device object ({:#x})", status);
        return status;
    }

    let symbolic_link = create_unicode_string(obfstr::wide!("\\??\\Eagle\0"));
    status = unsafe { IoCreateSymbolicLink(&symbolic_link, &device_name) };

    if !NT_SUCCESS(status) {
        log::error!("Failed to create symbolic link ({:#x})", status);
        return status;
    }

    //ProcessNotify (called when a process is created)
    unsafe { PsSetCreateProcessNotifyRoutineEx(process_create_callback as PcreateProcessNotifyRoutineEx, FALSE) };


    return STATUS_SUCCESS;
}


pub extern "system" fn dispatch_device_control(_device_object: &mut DEVICE_OBJECT, irp: &mut IRP) -> NTSTATUS {
    
    let stack = IoGetCurrentIrpStackLocation(irp);
    let control_code = unsafe { (*stack).Parameters.DeviceIoControl().IoControlCode };
    let mut status = STATUS_UNSUCCESSFUL;
    let mut byte_io: usize = 0;

    match control_code {
        IOCTL_PROCESS_PROTECT_REQUEST => {
            log::info!("IOCTL_PROCESS_PROTECT_REQUEST");
            let protect_process_status = protect_process(irp, stack);
           
            if NT_SUCCESS(protect_process_status) {
                log::info!("Process protection successful");
                byte_io = size_of::<TargetProcess>();
                status = STATUS_SUCCESS;
            } else {
                log::error!("Process protection failed");
            }
        },
        IOCTL_PROCESS_UNPROTECT_REQUEST => {
            log::info!("IOCTL_PROCESS_UNPROTECT_REQUEST");
            let unprotect_process_status = unprotect_process(irp, stack);
           
            if NT_SUCCESS(unprotect_process_status) {
                log::info!("Process unprotection successful");
                byte_io = size_of::<TargetProcess>();
                status = STATUS_SUCCESS;
            } else {
                log::error!("Process unprotection failed");
            }
        },
        IOCTL_PROCESS_TOKEN_PRIVILEGES_REQUEST => {
            log::info!("IOCTL_PROCESS_TOKEN_PRIVILEGES_REQUEST");
            let token_privs_status = enable_all_token_privileges(irp, stack);
           
            if NT_SUCCESS(token_privs_status) {
                log::info!("Process token privileges successful");
                byte_io = size_of::<TargetProcess>();
                status = STATUS_SUCCESS;
            } else {
                log::error!("Process token privileges failed");
            }
        },
        IOCTL_PROCESS_ENUM_CALLBACKS => {
            log::info!("IOCTL_PROCESS_ENUM_CALLBACKS");

            let buffer_size = (size_of::<CallBackInformation>() * 64) as u32;

            if unsafe { (*stack).Parameters.DeviceIoControl().OutputBufferLength } < buffer_size {
                status = STATUS_BUFFER_TOO_SMALL;
                log::error!("CallBackInformation buffer to small");
                return complete_request(irp, status, byte_io);
            }

            let user_buffer = irp.UserBuffer as *mut CallBackInformation;
            
            if user_buffer.is_null() {
                log::error!("CallBackInformation user buffer is null");
                status = STATUS_INVALID_PARAMETER;
                return complete_request(irp, status,byte_io);
            }

            // Get a a pointer to the array
            let psp_array_address = find_psp_set_create_process_notify();

            if psp_array_address.is_null() {
                log::error!("psp_array_address is null");
                status = STATUS_NOT_FOUND;
                return complete_request(irp, status, byte_io);
            }

            // THE MAGIC

            for i in 0..64 {
                // 64 bit addresses are 8 bytes (8*0, 8*1, 8*2, 8*3 etc...)
                let p_callback = unsafe { psp_array_address.offset(i * 8) };
                let callback = unsafe { *(p_callback as *const u64) };

                unsafe { (*user_buffer).pointer = callback };

                if callback > 0 {
                    log::info!("Calling search_loaded_modules");
                    unsafe { search_loaded_modules(user_buffer.cast::<u8>().offset(i) as *mut CallBackInformation) };
                }

                byte_io += size_of::<CallBackInformation>();
            }

            //THE END
            log::info!("Enumeration of modules complete");
            status = STATUS_SUCCESS;

        },
        _ => {
            log::error!("Invalid IOCTL code")
        },
    }

    unsafe { *(irp.IoStatus.__bindgen_anon_1.Status_mut()) = status };
    irp.IoStatus.Information = byte_io;
    irp.IoStatus.Information = 0;
    unsafe { IoCompleteRequest(irp, IO_NO_INCREMENT) };

    return STATUS_SUCCESS;
}

fn complete_request(irp: &mut IRP, status: NTSTATUS, byte_io: usize) -> NTSTATUS {
    unsafe { *(irp.IoStatus.__bindgen_anon_1.Status_mut()) = status };
    irp.IoStatus.Information = byte_io;
    irp.IoStatus.Information = 0;
    unsafe { IoCompleteRequest(irp, IO_NO_INCREMENT) };

    return status;
}

pub extern "system" fn dispatch_create_close(_device_object: &mut DEVICE_OBJECT, irp: &mut IRP) -> NTSTATUS {
    let stack = IoGetCurrentIrpStackLocation(irp);
    let code = unsafe { (*stack).MajorFunction };

	if code == IRP_MJ::CREATE as u8 {
		log::info!("IRP_MJ_CREATE called");
	} else {
		log::info!("IRP_MJ_CLOSE called");
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
    log::info!("Driver unloaded successfully!");
}