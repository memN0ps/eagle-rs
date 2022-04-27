use core::{mem::zeroed};
use alloc::vec::Vec;
use kernel_print::kernel_println;
use ntapi::{ntrtl::RtlInitUnicodeString, winapi::shared::ntdef::{PCWSTR, UNICODE_STRING, PUNICODE_STRING, PVOID}};

extern "system" {
    pub fn MmGetSystemRoutineAddress(SystemRoutineName: PUNICODE_STRING) -> PVOID;
}

fn get_ntoskrnl_exports(export_name: PCWSTR) -> PVOID {
    let mut file_name: UNICODE_STRING = unsafe { zeroed::<UNICODE_STRING>() };

    unsafe { RtlInitUnicodeString(&mut file_name, export_name); };

    //The MmGetSystemRoutineAddress routine returns a pointer to a function specified by SystemRoutineName.
    return unsafe { MmGetSystemRoutineAddress(&mut file_name) };
}

pub fn get_function_base_address(function_name: &str) -> PVOID {

    let mut unicode_function_name: Vec<_> =  function_name.encode_utf16().collect();
    unicode_function_name.push(0x0);
    
    let base = get_ntoskrnl_exports(unicode_function_name.as_ptr());

    return base;
}

#[allow(dead_code)]
pub fn get_eprocess_protection_offset() -> isize {
    let base_address = get_function_base_address("PsIsProtectedProcess");
    let function_bytes: &[u8] = unsafe { core::slice::from_raw_parts(base_address as *const u8, 5) };

    //kernel_println!("Function Bytes in Decimal: {:?}", function_bytes);

    let slice = &function_bytes[2..4];
    let protection_offset = u16::from_le_bytes(slice.try_into().unwrap());
    kernel_println!("[*] EPROCESS.PROTECTION OFFSET: {:#x}", protection_offset);

    return protection_offset as isize;
}

pub fn get_eprocess_signature_level_offset() -> isize {
    let base_address = get_function_base_address("PsGetProcessSignatureLevel");
    let function_bytes: &[u8] = unsafe { core::slice::from_raw_parts(base_address as *const u8, 20) };

    //kernel_println!("Function Bytes in Decimal: {:?}", function_bytes);

    let slice = &function_bytes[15..17];
    let signature_level_offset = u16::from_le_bytes(slice.try_into().unwrap());
    kernel_println!("[*] EPROCESS.SIGNATURE_LEVEL OFFSET: {:#x}", signature_level_offset);

    return signature_level_offset as isize;
}   

/*

0: kd> u PsIsProtectedProcess	
nt!PsIsProtectedProcess:
fffff807`0d87c730 f6817a08000007  test    byte ptr [rcx+87Ah],7
fffff807`0d87c737 b800000000      mov     eax,0
fffff807`0d87c73c 0f97c0          seta    al
fffff807`0d87c73f c3              ret
fffff807`0d87c740 cc              int     3
fffff807`0d87c741 cc              int     3
fffff807`0d87c742 cc              int     3
fffff807`0d87c743 cc              int     3

0: kd> u PsGetProcessSignatureLevel	
nt!PsGetProcessSignatureLevel:
fffff807`0d992dd0 4885d2          test    rdx,rdx
fffff807`0d992dd3 7408            je      nt!PsGetProcessSignatureLevel+0xd (fffff807`0d992ddd)
fffff807`0d992dd5 8a8179080000    mov     al,byte ptr [rcx+879h]
fffff807`0d992ddb 8802            mov     byte ptr [rdx],al
fffff807`0d992ddd 8a8178080000    mov     al,byte ptr [rcx+878h]
fffff807`0d992de3 c3              ret
fffff807`0d992de4 cc              int     3
fffff807`0d992de5 cc              int     3

nt!_EPROCESS
        +0x878 SignatureLevel   : UChar
        +0x879 SectionSignatureLevel : UChar
        +0x87a Protection       : _PS_PROTECTION

*/