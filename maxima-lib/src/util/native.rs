use std::{
    ffi::{CString, OsString},
    path::PathBuf,
    ptr::null_mut,
};

use anyhow::{bail, Result};

#[cfg(target_family = "windows")]
use std::os::windows::prelude::OsStringExt;

#[cfg(target_family = "windows")]
use winapi::{
    shared::windef::HWND,
    um::{
        libloaderapi::{GetModuleFileNameW, GetModuleHandleW},
        wincon::GetConsoleWindow,
        winuser::{
            EnumWindows, FindWindowA, GetWindowThreadProcessId, IsWindowVisible,
            SetForegroundWindow,
        },
    },
};

#[cfg(target_family = "windows")]
unsafe extern "system" fn enum_windows_proc(
    hwnd: HWND,
    _l_param: winapi::shared::minwindef::LPARAM,
) -> winapi::shared::minwindef::BOOL {
    let mut window_process_id: u32 = 0;

    GetWindowThreadProcessId(hwnd, &mut window_process_id);

    if window_process_id != std::process::id() || IsWindowVisible(hwnd) == 0 {
        return winapi::shared::minwindef::TRUE;
    }

    if IsWindowVisible(hwnd) != 0 {
        SetForegroundWindow(hwnd);
    }

    winapi::shared::minwindef::TRUE
}
#[cfg(target_family = "windows")]
pub fn get_hwnd() -> Result<HWND> {
    unsafe {
        EnumWindows(Some(enum_windows_proc), 0);

        let window_name = CString::new("Maxima").expect("Failed to create native string");
        let mut hwnd = FindWindowA(std::ptr::null(), window_name.as_ptr());
        if !hwnd.is_null() {
            println!("Is not null");
            Ok(hwnd)
        } else {
            hwnd = GetConsoleWindow();
            if !hwnd.is_null() {
                //bail!("Failed to find native window");
                Ok(hwnd)
            } else {
                bail!("Failed to find native window");
            }
        }
    }
}

#[cfg(target_family = "windows")]
pub fn take_foreground_focus() -> Result<()> {
    unsafe {
        EnumWindows(Some(enum_windows_proc), 0);
    }

    Ok(())
}

#[cfg(target_family = "unix")]
pub fn take_foreground_focus() -> Result<()> {
    todo!();
}

#[cfg(target_family = "windows")]
pub fn get_module_path() -> Result<PathBuf> {
    // Get a handle to the DLL
    let hmodule = unsafe { GetModuleHandleW(null_mut()) };
    if hmodule.is_null() {
        bail!("Failed to find module");
    }

    // Create a buffer to hold the DLL path
    let mut buffer: [u16; 260] = [0; 260];

    // Get the DLL path
    let length = unsafe { GetModuleFileNameW(hmodule, buffer.as_mut_ptr(), buffer.len() as u32) };
    if length == 0 {
        bail!("Failed to get module length");
    }

    // Convert buffer to a Rust String
    let os_string = OsString::from_wide(&buffer[0..length as usize]);
    Ok(os_string.to_string_lossy().into_owned().into())
}

#[cfg(target_family = "unix")]
pub fn get_module_path() -> Option<String> {
    todo!();
}
