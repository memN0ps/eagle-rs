use core::ptr::slice_from_raw_parts;
use alloc::{string::String};
use winapi::{shared::{ntdef::{HANDLE, BOOLEAN, NTSTATUS}}, km::wdm::{PEPROCESS}};
use crate::includes::PS_CREATE_NOTIFY_INFO;

#[allow(non_snake_case)]
pub type PcreateProcessNotifyRoutineEx = extern "system" fn(Process: PEPROCESS, ProcessId: HANDLE, CreateInfo: *mut PS_CREATE_NOTIFY_INFO);
//type PcreateProcessNotifyRoutine = extern "system" fn(ParentId: HANDLE, ProcessId: HANDLE, Create: BOOLEAN);
//type PcreateThreadNotifyRoutine = extern "system" fn(ProcessId: HANDLE, ThreadId: HANDLE, Create: BOOLEAN);
//type PloadImageNotifyRoutine = extern "system" fn(FullImageName: PUNICODE_STRING, ProcessId: HANDLE , ImageInfo: *mut IMAGE_INFO);

extern "system" {
    #[allow(non_snake_case)]
    pub fn PsSetCreateProcessNotifyRoutineEx(NotifyRoutine: PcreateProcessNotifyRoutineEx, Remove: BOOLEAN) -> NTSTATUS;    
    //pub fn PsSetCreateProcessNotifyRoutine(NotifyRoutine: PcreateProcessNotifyRoutine, Remove: BOOLEAN) -> NTSTATUS;    
    //pub fn PsSetCreateThreadNotifyRoutine(NotifyRoutine: PcreateThreadNotifyRoutine) -> NTSTATUS;
    //pub fn PsSetLoadImageNotifyRoutine(NotifyRoutine: PloadImageNotifyRoutine) -> NTSTATUS;
}

#[allow(non_snake_case)]
pub extern "system" fn process_create_callback(_Process: PEPROCESS, ProcessId: HANDLE, CreateInfo: *mut PS_CREATE_NOTIFY_INFO) {
    if !CreateInfo.is_null() {
        let file_open = unsafe { (*CreateInfo).param0.param0.FileOpenNameAvailable };

        if file_open != 0 {
            let p_str = unsafe { *(*CreateInfo).ImageFileName };
            let slice = unsafe { &*slice_from_raw_parts(p_str.Buffer, p_str.Length as usize / 2) } ;
            let process_name = String::from_utf16(slice).unwrap();
            let process_id = ProcessId as u32;
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

