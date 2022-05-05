use core::{ptr::{slice_from_raw_parts, null_mut}, mem::size_of, intrinsics::copy_nonoverlapping};
use alloc::{string::String};
use common::CallBackInformation;
use kernel_alloc::nt::{ExAllocatePool, ExFreePool};
use winapi::{shared::{ntdef::{HANDLE, BOOLEAN, NTSTATUS, NT_SUCCESS}, ntstatus::{STATUS_UNSUCCESSFUL, STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES}}, km::wdm::{PEPROCESS}, um::winnt::RtlZeroMemory, ctypes::c_void};
use crate::includes::{PSCreateNotifyInfo, AuxKlibInitialize, AuxKlibQueryModuleInformation, AuxModuleExtendedInfo};

#[allow(non_snake_case)]
pub type PcreateProcessNotifyRoutineEx = extern "system" fn(process: PEPROCESS, process_id: HANDLE, create_info: *mut PSCreateNotifyInfo);
//type PcreateProcessNotifyRoutine = extern "system" fn(ParentId: HANDLE, ProcessId: HANDLE, Create: BOOLEAN);
//type PcreateThreadNotifyRoutine = extern "system" fn(ProcessId: HANDLE, ThreadId: HANDLE, Create: BOOLEAN);
//type PloadImageNotifyRoutine = extern "system" fn(FullImageName: PUNICODE_STRING, ProcessId: HANDLE , ImageInfo: *mut IMAGE_INFO);

extern "system" {
    #[allow(non_snake_case)]
    pub fn PsSetCreateProcessNotifyRoutineEx(notify_routine: PcreateProcessNotifyRoutineEx, remove: BOOLEAN) -> NTSTATUS;    
    //pub fn PsSetCreateProcessNotifyRoutine(notify_routine: PcreateProcessNotifyRoutine, remove: BOOLEAN) -> NTSTATUS;    
    //pub fn PsSetCreateThreadNotifyRoutine(notify_routine: PcreateThreadNotifyRoutine) -> NTSTATUS;
    //pub fn PsSetLoadImageNotifyRoutine(notify_routine: PloadImageNotifyRoutine) -> NTSTATUS;
}

#[allow(non_snake_case)]
pub extern "system" fn process_create_callback(_process: PEPROCESS, process_id: HANDLE, create_info: *mut PSCreateNotifyInfo) {
    if !create_info.is_null() {
        let file_open = unsafe { (*create_info).param0.param0.file_open_available };

        if file_open != 0 {
            let p_str = unsafe { *(*create_info).image_file_name };
            let slice = unsafe { &*slice_from_raw_parts(p_str.Buffer, p_str.Length as usize / 2) } ;
            let process_name = String::from_utf16(slice).unwrap();
            let process_id = process_id as u32;
            log::info!("Process Created: {:?} ({:?})", process_name, process_id);
        }
    }
}

/*
PsSetCreateProcessNotifyRoutineEx: https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/ntddk/nf-ntddk-pssetcreateprocessnotifyroutineex
PsSetCreateProcessNotifyRoutine: https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/ntddk/nf-ntddk-pssetcreateprocessnotifyroutine
PsSetCreateThreadNotifyRoutine: https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/ntddk/nf-ntddk-pssetcreateprocessnotifyroutine
PsSetLoadImageNotifyRoutine: https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/ntddk/nf-ntddk-pssetloadimagenotifyroutine

PcreateProcessNotifyRoutineEx: https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/ntddk/nc-ntddk-pcreate_process_notify_routine_ex
PcreateProcessNotifyRoutine: https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/ntddk/nc-ntddk-pcreate_process_notify_routine
PcreateThreadNotifyRoutine: https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/ntddk/nc-ntddk-pcreate_thread_notify_routine
PloadImageNotifyRoutine: https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/ntddk/nc-ntddk-pload_image_notify_routine
*/


/*
AuxKlibInitialize: https://docs.microsoft.com/en-us/windows/win32/devnotes/auxklibinitialize-func
AuxKlibQueryModuleInformation: https://docs.microsoft.com/en-us/windows/win32/devnotes/auxklibquerymoduleinformation-func
*/

/// Allocate
pub fn search_loaded_modules(module_info: *mut CallBackInformation) -> NTSTATUS {

    log::info!("Calling AuxKlibInitialize");
    let status = unsafe { AuxKlibInitialize() };

    if !NT_SUCCESS(status) {
        log::error!("Failed to call AuxKlibInitialize ({:#x})", status);
        return STATUS_UNSUCCESSFUL;
    }

    let mut buffer_size: u32 = 0;

    //1st: get the buffer size required to hold the requested information
    log::info!("Calling 1st AuxKlibQueryModuleInformation");
    let status = unsafe { AuxKlibQueryModuleInformation(&mut buffer_size, size_of::<AuxModuleExtendedInfo>() as u32, null_mut()) };

    if !NT_SUCCESS(status) {
        log::error!("1st AuxKlibQueryModuleInformation failed ({:#x})", status);
        return STATUS_UNSUCCESSFUL;
    }

    // allocate memory
    log::info!("Calling ExAllocatePool");
    let modules = unsafe { ExAllocatePool(kernel_alloc::nt::PoolType::NonPagedPool, buffer_size as usize) as *mut c_void };

    if modules.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // Zero out the memory location to be filled.
    log::info!("Calling RtlZeroMemory");
    unsafe { RtlZeroMemory(modules, buffer_size as usize) };

    //2nd: get the information
    log::info!("Calling 2nd AuxKlibQueryModuleInformation");
    let status = unsafe { AuxKlibQueryModuleInformation(&mut buffer_size, size_of::<AuxModuleExtendedInfo>() as u32, modules) };

    if !NT_SUCCESS(status) {
        log::error!("2nd AuxKlibQueryModuleInformation failed ({:#x})", status);
        return STATUS_UNSUCCESSFUL;
    }

    // do the magic
    log::info!("Getting number of modules");
    let number_of_modules = buffer_size / size_of::<AuxModuleExtendedInfo>() as u32;

    let module = modules as *mut AuxModuleExtendedInfo;
    
    log::info!("Looping through the number of modules");
    for i  in 0..number_of_modules {
        let start_address = unsafe { (*module.offset(i as isize)).basic_info.image_base };
        let image_size = unsafe { (*module.offset(i as isize)).image_size };
        let end_address = unsafe { start_address.cast::<u8>().offset(image_size as isize) as u64 };

        let raw_pointer = unsafe { *(((*module_info).pointer &  0xfffffffffffffff8) as * mut u64) };

        if raw_pointer > start_address as u64 && raw_pointer < end_address {
            let mut dst = unsafe { (*module_info).module_name };
            
            let src = unsafe { 
                (*module.offset(i as isize)).full_path_name[(*module.offset(i as isize)).file_name_offset as usize] as *const u8
            };
            
            unsafe { copy_nonoverlapping(src, dst.as_mut_ptr(), 256) };
        }
    }


    unsafe { ExFreePool(modules as u64) };

    return STATUS_SUCCESS;
}