use core::{ptr::{null_mut, null}, mem::{zeroed, size_of}};

use kernel_alloc::nt::{ExAllocatePool};
use winapi::{shared::{ntdef::{NT_SUCCESS}}, ctypes::c_void, um::winnt::RtlZeroMemory};

use crate::{includes::{strstr}, includes::SystemInformationClass, includes::SystemModuleInformation, includes::ZwQuerySystemInformation};

pub fn get_module_base(module_name: *const i8) -> *mut c_void {

    let mut bytes = 0;

    // Get buffer size
    let status = unsafe { ZwQuerySystemInformation(
        SystemInformationClass::SystemModuleInformation,
            null_mut(),
            0,
            &mut bytes
        ) 
    };

    log::info!("1st ZwQuerySystemInformation Bytes: {:?}", bytes);

    /*
    if !NT_SUCCESS(status) {
        log::error!("[-] 1st ZwQuerySystemInformation failed {:?}", status);
        return null_mut();
    } */

    let module_info = unsafe { 
        ExAllocatePool(kernel_alloc::nt::PoolType::NonPagedPool, bytes as usize) as *mut SystemModuleInformation 
    };

    if module_info.is_null() {
        log::error!("[-] ExAllocatePool failed");
        return null_mut();
    }

    unsafe { RtlZeroMemory( module_info as *mut c_void, bytes as usize) };

    let status = unsafe { ZwQuerySystemInformation(
        SystemInformationClass::SystemModuleInformation,
        module_info as *mut c_void,
        bytes,
        &mut bytes) 
    };

    log::info!("2nd ZwQuerySystemInformation Bytes: {:?}", bytes);

    if !NT_SUCCESS(status) {
        log::info!("[-] 2nd ZwQuerySystemInformation failed {:#x}", status);
        return null_mut();
    }


    let mut p_module: *mut c_void = null_mut();
    log::info!("Modules 1: {:?}", p_module);


    for i in unsafe { 0..(*module_info).modules_count as usize } {

        let image_name = unsafe { (*module_info).modules[i].image_name };
        let image_base = unsafe { (*module_info).modules[i].image_base };

        if unsafe { strstr(image_name.as_ptr(), module_name as *const u8) != null() } {
            log::info!("[+] Module name: {:?} and module base: {:?}", image_name, image_base);
            p_module = image_base;
            break;
        }
    }
    log::info!("Modules 2: {:?}", p_module);

    return p_module;
}

/* 
fn get_ci_options_offset() {
    let dll_name = CString::new("CI.dll").unwrap();

    let k_module_base = get_module_base(dll_name.as_ptr());
    let module_base = unsafe { LoadLibraryExA(dll_name.as_ptr(), NULL, DONT_RESOLVE_DLL_REFERENCES) };

    if module_base.is_null() {
        panic!("[-] LoadLibraryExA failed")
    }

    let function_name = CString::new("CiInitialize").unwrap();
    let ci_initialize = unsafe { GetProcAddress(module_base, function_name.as_ptr()) };

    if ci_initialize.is_null() {
        panic!("[-] GetProcAddress failed");
    }

    println!("[+] CI!CiInitialize: {:?}", ci_initialize);

    let ci_initialize_offset = ci_initialize as usize - module_base as usize;
    println!("[+] CI!CiInitialize offset: {:#x}", ci_initialize_offset);

    let k_ci_initialize = ci_initialize_offset + k_module_base as usize;
    println!("[+] Kernel CI!CiInitialize: {:#x}", k_ci_initialize);
}*/