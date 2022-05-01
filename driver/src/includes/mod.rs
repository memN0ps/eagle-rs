use winapi::{shared::{ntdef::{HANDLE, BOOLEAN, NTSTATUS, ULONG, PVOID, PCUNICODE_STRING, UNICODE_STRING, LARGE_INTEGER, LIST_ENTRY, CSHORT}, basetsd::SIZE_T, minwindef::USHORT}, km::wdm::{KEVENT, KSPIN_LOCK, PDEVICE_OBJECT, PEPROCESS}, um::winnt::PACCESS_TOKEN};

extern "system" {
    #[allow(dead_code)]
    pub fn MmIsAddressValid(virtual_address: PVOID) -> bool;

    pub fn PsLookupProcessByProcessId(process_id: HANDLE, process: *mut PEPROCESS) -> NTSTATUS;

    pub fn PsReferencePrimaryToken(process: PEPROCESS) -> PACCESS_TOKEN;

    pub fn PsDereferencePrimaryToken(primary_token: PACCESS_TOKEN);

    pub fn ObfDereferenceObject(object: PVOID);

    pub fn MmGetSystemRoutineAddress(system_routine_name: *mut UNICODE_STRING) -> PVOID;
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
#[derive(Debug, Clone, Copy)]
pub struct ClientID {
    pub unique_process: HANDLE,
    pub unique_thread: HANDLE,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
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
#[derive(Clone, Copy)]
pub union ImageInfo0 {
    pub properties: ULONG,
    pub param0: ImageInfoProperties,
}

#[repr(C)]
pub struct ImageInfo {
    pub param0: ImageInfo0,    
    pub image_base: PVOID,
    pub image_selector: ULONG,
    pub image_size: SIZE_T,
    pub image_section_number: ULONG,
}

#[repr(C)]
pub struct VPB {
    pub ttype: CSHORT,
    pub size: CSHORT,
    pub flags: USHORT,
    pub volume_label_length: USHORT,
    pub device_object: PDEVICE_OBJECT,
    pub real_device: PDEVICE_OBJECT,
    pub serial_number: ULONG,
    pub reference_count: ULONG,
    pub volume_label: u16,
}

#[repr(C)]
pub struct IoCompletionContext {
    pub port: PVOID,
    pub key: PVOID,
}

#[repr(C)]
pub struct SectionObjectPointers {
    pub data_section_object: PVOID,
    pub shared_cache_map: PVOID,
    pub image_section_object: PVOID,
}

#[repr(C)]
pub struct FileObject {
    pub ttype: CSHORT,
    pub size: CSHORT,
    pub device_object: PDEVICE_OBJECT,
    pub vpb: *mut VPB,
    pub fs_context: PVOID,
    pub fs_context32: PVOID,
    pub section_object_pointer: *mut SectionObjectPointers,
    pub private_cache_map: PVOID,
    pub final_status: NTSTATUS,
    pub related_file_object: *mut FileObject,
    pub lock_operation: BOOLEAN,
    pub delete_pending: BOOLEAN,
    pub read_access: BOOLEAN,
    pub write_access: BOOLEAN,
    pub delete_access: BOOLEAN,
    pub shared_read: BOOLEAN,
    pub shared_write: BOOLEAN,
    pub shared_delete: BOOLEAN,
    pub flags: ULONG,
    pub file_name: UNICODE_STRING,
    pub current_byte_offset: LARGE_INTEGER,
    pub waiters: ULONG,
    pub busy: ULONG,
    pub last_lock: PVOID,
    pub lock: KEVENT,
    pub event: KEVENT,
    pub completion_context: *mut IoCompletionContext,
    pub irp_list_lock: KSPIN_LOCK,
    pub irp_list: LIST_ENTRY,
    pub file_object_extension: PVOID,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PSCreateNotifyInfo00 {
    pub file_open_available: ULONG,
    pub is_subsystem_process: ULONG,
    pub reserved: ULONG,
}

#[repr(C)]
pub union PSCreateNotifyInfo0 {
    pub flags: ULONG,
    pub param0: PSCreateNotifyInfo00,
}

#[repr(C)]
pub struct PSCreateNotifyInfo {
    pub size: SIZE_T,
    pub param0: PSCreateNotifyInfo0,
    pub parent_process_id: HANDLE,
    pub creating_thread_id: ClientID,
    pub file_object: *mut FileObject,
    pub image_file_name: PCUNICODE_STRING,
    pub command_line: PCUNICODE_STRING,
    pub creation_status: NTSTATUS,
}