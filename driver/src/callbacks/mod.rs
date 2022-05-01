use core::ptr::slice_from_raw_parts;
use alloc::{string::String};
use winapi::{shared::{ntdef::{HANDLE, BOOLEAN, NTSTATUS}}, km::wdm::{PEPROCESS}};
use crate::{includes::PSCreateNotifyInfo};

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

