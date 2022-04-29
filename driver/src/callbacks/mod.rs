use core::ptr::slice_from_raw_parts;

use alloc::{vec::Vec, string::String};
use kernel_print::{kernel_println};
use winapi::{shared::{ntdef::{HANDLE, BOOLEAN, NTSTATUS, ULONG, PVOID, PCUNICODE_STRING, UNICODE_STRING, LARGE_INTEGER, LIST_ENTRY, CSHORT}, basetsd::SIZE_T, minwindef::USHORT, ntstatus::{STATUS_UNSUCCESSFUL, STATUS_SUCCESS}}, km::wdm::{KEVENT, KSPIN_LOCK, PDEVICE_OBJECT, PEPROCESS}};

#[repr(C)]
pub struct CLIENT_ID {
    pub UniqueProcess: HANDLE,
    pub UniqueThread: HANDLE,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ImageInfoProperties {
    image_address_mode: ULONG,
    system_mode_image: ULONG,
    image_mapped_to_all_pids: ULONG,
    extended_info_present: ULONG,
    machine_type_mismatch: ULONG,
    image_signature_level: ULONG,
    image_signature_type: ULONG,
    image_partial_map: ULONG,
    reserved: ULONG,
}

#[repr(C)]
pub union IMAGE_INFO_0 {
    properties: ULONG,
    param0: ImageInfoProperties,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct IMAGE_INFO {
    param0: IMAGE_INFO_0,    
    image_base: PVOID,
    image_selector: ULONG,
    image_size: SIZE_T,
    image_section_number: ULONG,
}

//test
pub const MAXIMUM_VOLUME_LABEL_LENGTH: u32 = 32;
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct VPB {
    Type: CSHORT,
    Size: CSHORT,
    Flags: USHORT,
    VolumeLabelLength: USHORT,
    DeviceObject: PDEVICE_OBJECT,
    RealDevice: PDEVICE_OBJECT,
    SerialNumber: ULONG,
    ReferenceCount: ULONG,
    VolumeLabel: u16,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct IO_COMPLETION_CONTEXT {
    Port: PVOID,
    Key: PVOID,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct SECTION_OBJECT_POINTERS {
    DataSectionObject: PVOID,
    SharedCacheMap: PVOID,
    ImageSectionObject: PVOID,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct FILE_OBJECT {
    Type: CSHORT,
    Size: CSHORT,
    DeviceObject: PDEVICE_OBJECT,
    Vpb: *mut VPB,
    FsContext: PVOID,
    FsContext32: PVOID,
    SectionObjectPointer: *mut SECTION_OBJECT_POINTERS,
    PrivateCacheMap: PVOID,
    FinalStatus: NTSTATUS,
    RelatedFileObject: *mut FILE_OBJECT,
    LockOperation: BOOLEAN,
    DeletePending: BOOLEAN,
    ReadAccess: BOOLEAN,
    WriteAccess: BOOLEAN,
    DeleteAccess: BOOLEAN,
    SharedRead: BOOLEAN,
    SharedWrite: BOOLEAN,
    SharedDelete: BOOLEAN,
    Flags: ULONG,
    FileName: UNICODE_STRING,
    CurrentByteOffset: LARGE_INTEGER,
    Waiters: ULONG,
    Busy: ULONG,
    LastLock: PVOID,
    Lock: KEVENT,
    Event: KEVENT,
    CompletionContext: *mut IO_COMPLETION_CONTEXT,
    IrpListLock: KSPIN_LOCK,
    IrpList: LIST_ENTRY,
    FileObjectExtension: PVOID,
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub struct PS_CREATE_NOTIFY_INFO_0_0 {
    FileOpenNameAvailable: ULONG,
    IsSubsystemProcess: ULONG,
    Reserved: ULONG,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub union PS_CREATE_NOTIFY_INFO_0 {
    Flags: ULONG,
    param0: PS_CREATE_NOTIFY_INFO_0_0,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct PS_CREATE_NOTIFY_INFO {
    Size: SIZE_T,
    param0: PS_CREATE_NOTIFY_INFO_0,
    ParentProcessId: HANDLE,
    CreatingThreadId: CLIENT_ID,
    FileObject: *mut FILE_OBJECT,
    ImageFileName: PCUNICODE_STRING,
    CommandLine: PCUNICODE_STRING,
    CreationStatus: NTSTATUS,
}

//test

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
pub extern "system" fn process_create_callback(Process: PEPROCESS, ProcessId: HANDLE, CreateInfo: *mut PS_CREATE_NOTIFY_INFO) {
    if !CreateInfo.is_null() {
        let file_open = unsafe { (*CreateInfo).param0.param0.FileOpenNameAvailable };

        if file_open != 0 {
            let p_str = unsafe { *(*CreateInfo).ImageFileName };
            let slice = unsafe { &*slice_from_raw_parts(p_str.Buffer, p_str.Length as usize / 2) } ;
            let process_name = String::from_utf16(slice).unwrap();
            let process_id = ProcessId as u32;
            kernel_println!("[+] Process Created: {:?} ({:?})", process_name, process_id);
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

