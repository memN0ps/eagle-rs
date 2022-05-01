use winapi::{shared::{ntdef::{HANDLE, BOOLEAN, NTSTATUS, ULONG, PVOID, PCUNICODE_STRING, UNICODE_STRING, LARGE_INTEGER, LIST_ENTRY, CSHORT}, basetsd::SIZE_T, minwindef::USHORT}, km::wdm::{KEVENT, KSPIN_LOCK, PDEVICE_OBJECT, PEPROCESS}, um::winnt::PACCESS_TOKEN};

extern "system" {
    #[allow(dead_code)]
    pub fn MmIsAddressValid(virtual_address: PVOID) -> bool;

    pub fn PsLookupProcessByProcessId(process_id: HANDLE, process: *mut PEPROCESS) -> NTSTATUS;

    pub fn PsReferencePrimaryToken(process: PEPROCESS) -> PACCESS_TOKEN;

    pub fn PsDereferencePrimaryToken(PrimaryToken: PACCESS_TOKEN);

    pub fn ObfDereferenceObject(object: PVOID);

    pub fn MmGetSystemRoutineAddress(SystemRoutineName: *mut UNICODE_STRING) -> PVOID;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ProcessPrivileges {
    pub present: [u8; 8],
    pub enabled: [u8; 8],
    pub enabled_by_default: [u8; 8],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PSProtection {
    pub protection_type: u8,
    pub protection_audit: u8,
    pub protection_signer: u8,
}

impl Default for PSProtection {
    fn default() -> Self {
        Self { protection_type: 3, protection_audit: 1, protection_signer: 4 }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ProcessProtectionInformation {
    pub signature_level: u8,
	pub section_signature_level: u8,
	pub protection: PSProtection,
}


#[repr(C)]
#[allow(non_snake_case)]
pub struct CLIENT_ID {
    pub UniqueProcess: HANDLE,
    pub UniqueThread: HANDLE,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ImageInfoProperties {
    pub image_address_mode: ULONG,
    pub system_mode_image: ULONG,
    pub image_mapped_to_all_pids: ULONG,
    pub extended_info_present: ULONG,
    pub machine_type_mismatch: ULONG,
    pub image_signature_level: ULONG,
    pub image_signature_type: ULONG,
    pub image_partial_map: ULONG,
    pub reserved: ULONG,
}

#[repr(C)]
pub union IMAGE_INFO_0 {
    pub properties: ULONG,
    pub param0: ImageInfoProperties,
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
pub struct IMAGE_INFO {
    pub param0: IMAGE_INFO_0,    
    pub image_base: PVOID,
    pub image_selector: ULONG,
    pub image_size: SIZE_T,
    pub image_section_number: ULONG,
}

#[allow(dead_code)]
#[repr(C)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct VPB {
    pub Type: CSHORT,
    pub Size: CSHORT,
    pub Flags: USHORT,
    pub VolumeLabelLength: USHORT,
    pub DeviceObject: PDEVICE_OBJECT,
    pub RealDevice: PDEVICE_OBJECT,
    pub SerialNumber: ULONG,
    pub ReferenceCount: ULONG,
    pub VolumeLabel: u16,
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct IO_COMPLETION_CONTEXT {
    pub Port: PVOID,
    pub Key: PVOID,
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct SECTION_OBJECT_POINTERS {
    pub DataSectionObject: PVOID,
    pub SharedCacheMap: PVOID,
    pub ImageSectionObject: PVOID,
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct FILE_OBJECT {
    pub Type: CSHORT,
    pub Size: CSHORT,
    pub DeviceObject: PDEVICE_OBJECT,
    pub Vpb: *mut VPB,
    pub FsContext: PVOID,
    pub FsContext32: PVOID,
    pub SectionObjectPointer: *mut SECTION_OBJECT_POINTERS,
    pub PrivateCacheMap: PVOID,
    pub FinalStatus: NTSTATUS,
    pub RelatedFileObject: *mut FILE_OBJECT,
    pub LockOperation: BOOLEAN,
    pub DeletePending: BOOLEAN,
    pub ReadAccess: BOOLEAN,
    pub WriteAccess: BOOLEAN,
    pub DeleteAccess: BOOLEAN,
    pub SharedRead: BOOLEAN,
    pub SharedWrite: BOOLEAN,
    pub SharedDelete: BOOLEAN,
    pub Flags: ULONG,
    pub FileName: UNICODE_STRING,
    pub CurrentByteOffset: LARGE_INTEGER,
    pub Waiters: ULONG,
    pub Busy: ULONG,
    pub LastLock: PVOID,
    pub Lock: KEVENT,
    pub Event: KEVENT,
    pub CompletionContext: *mut IO_COMPLETION_CONTEXT,
    pub IrpListLock: KSPIN_LOCK,
    pub IrpList: LIST_ENTRY,
    pub FileObjectExtension: PVOID,
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[derive(Debug, Clone, Copy)]
pub struct PS_CREATE_NOTIFY_INFO_0_0 {
    pub FileOpenNameAvailable: ULONG,
    pub IsSubsystemProcess: ULONG,
    pub Reserved: ULONG,
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub union PS_CREATE_NOTIFY_INFO_0 {
    pub Flags: ULONG,
    pub param0: PS_CREATE_NOTIFY_INFO_0_0,
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct PS_CREATE_NOTIFY_INFO {
    pub Size: SIZE_T,
    pub param0: PS_CREATE_NOTIFY_INFO_0,
    pub ParentProcessId: HANDLE,
    pub CreatingThreadId: CLIENT_ID,
    pub FileObject: *mut FILE_OBJECT,
    pub ImageFileName: PCUNICODE_STRING,
    pub CommandLine: PCUNICODE_STRING,
    pub CreationStatus: NTSTATUS,
}