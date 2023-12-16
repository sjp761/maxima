#[cfg(windows)]
use winapi::{
    ctypes::c_void,
    shared::{
        iprtrmib::TCP_TABLE_OWNER_PID_ALL,
        minwindef::{DWORD, FALSE},
        tcpmib::{MIB_TCPROW_OWNER_PID, MIB_TCPTABLE_OWNER_PID},
        winerror::{ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS},
        ws2def::AF_INET,
    },
    um::iphlpapi::GetExtendedTcpTable,
};

use anyhow::Result;

/// Finds a process that has an open TCP connection from local_port.
#[cfg(windows)]
pub fn get_process_id(local_port: u16) -> Result<Option<u32>> {
    use anyhow::bail;
    use std::{mem, ptr};

    unsafe {
        let mut table_size: DWORD = 0;
        let mut result = GetExtendedTcpTable(
            ptr::null_mut(),
            &mut table_size,
            FALSE,
            AF_INET as DWORD,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );

        if result != ERROR_INSUFFICIENT_BUFFER {
            bail!("Insufficient buffe for the TCP table");
        }

        let mut table_buffer: Vec<u8> = vec![0; table_size as usize];
        let table_ptr = table_buffer.as_mut_ptr() as *mut c_void;

        result = GetExtendedTcpTable(
            table_ptr,
            &mut table_size,
            FALSE,
            AF_INET as DWORD,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );

        if result != ERROR_SUCCESS {
            bail!("Failed to retrieve the TCP table: {}", result);
        }

        let table = table_ptr as *mut MIB_TCPTABLE_OWNER_PID;
        let num_entries = (*table).dwNumEntries;

        for i in 0..num_entries {
            let row = &*(((table_ptr as usize)
                + mem::size_of::<DWORD>()
                + i as usize * mem::size_of::<MIB_TCPROW_OWNER_PID>())
                as *const MIB_TCPROW_OWNER_PID);

            let pid = row.dwOwningPid;
            if pid == 0 || u16::from_be(row.dwLocalPort as u16) != local_port {
                continue;
            }

            return Ok(Some(pid));
        }

        Ok(None)
    }
}

#[cfg(unix)]
pub fn get_process_id(_local_port: u16) -> Result<Option<u32>> {
    Ok(Some(0)) // TODO
}
