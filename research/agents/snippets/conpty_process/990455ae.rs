#![allow(non_snake_case)]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr::{null, null_mut};

use windows::{
    core::{Result, Error, PCWSTR, PWSTR},
    Win32::{
        Foundation::{CloseHandle, HANDLE, WAIT_EVENT},
        System::{
            Console::{COORD, HPCON},
            Pipes::CreatePipe,
            Threading::{
                CreateProcessW, GetExitCodeProcess, GetProcessId,
                InitializeProcThreadAttributeList, UpdateProcThreadAttribute, WaitForSingleObject,
                CREATE_UNICODE_ENVIRONMENT, EXTENDED_STARTUPINFO_PRESENT, INFINITE,
                LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_INFORMATION, STARTF_USESTDHANDLES,
                STARTUPINFOEXW,
            },
        },
    },
};

/// Creates an anonymous pipe pair for use with pseudo console.
fn create_pipe_pair() -> Result<(HANDLE, HANDLE)> {
    let mut read_handle = HANDLE::default();
    let mut write_handle = HANDLE::default();
    unsafe { CreatePipe(&mut read_handle, &mut write_handle, None, 0)? };
    Ok((read_handle, write_handle))
}

/// Creates a pseudo console with the specified size.
fn create_pseudo_console(size: COORD) -> Result<(HPCON, HANDLE, HANDLE)> {
    let (pty_in, con_writer) = create_pipe_pair()?;
    let (con_reader, pty_out) = create_pipe_pair()?;

    // Create the pseudo console
    let console =
        unsafe { windows::Win32::System::Console::CreatePseudoConsole(size, pty_in, pty_out, 0)? };

    // Close the PTY-end of the pipes since they're now owned by the pseudo console
    unsafe {
        CloseHandle(pty_in)?;
        CloseHandle(pty_out)?;
    }

    Ok((console, con_reader, con_writer))
}

/// Initializes STARTUPINFOEXW with pseudo console attribute.
fn initialize_startup_info(console: &HPCON) -> Result<STARTUPINFOEXW> {
    let mut si_ex = STARTUPINFOEXW::default();
    si_ex.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
    si_ex.StartupInfo.hStdInput = HANDLE::default();
    si_ex.StartupInfo.hStdOutput = HANDLE::default();
    si_ex.StartupInfo.hStdError = HANDLE::default();
    si_ex.StartupInfo.dwFlags |= STARTF_USESTDHANDLES;

    let mut size = 0;
    unsafe {
        InitializeProcThreadAttributeList(
            Some(LPPROC_THREAD_ATTRIBUTE_LIST(null_mut())),
            1,
            Some(0),
            &mut size,
        )?;
    }

    let attribute_list = vec![0u8; size].into_boxed_slice();
    let attribute_list = Box::leak(attribute_list);
    si_ex.lpAttributeList = LPPROC_THREAD_ATTRIBUTE_LIST(attribute_list.as_mut_ptr() as _);

    unsafe {
        InitializeProcThreadAttributeList(Some(si_ex.lpAttributeList), 1, Some(0), &mut size)?;
        UpdateProcThreadAttribute(
            si_ex.lpAttributeList,
            0,
            0x00020016, // PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE
            Some(console.0 as *const _),
            std::mem::size_of::<HPCON>(),
            None,
            None,
        )?;
    }

    Ok(si_ex)
}

/// Creates a process attached to the pseudo console.
fn create_process_attached_to_conpty(
    command: &str,
    startup_info: &STARTUPINFOEXW,
) -> Result<PROCESS_INFORMATION> {
    let mut command_utf16: Vec<u16> = OsStr::new(command)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let commandline = PWSTR(command_utf16.as_mut_ptr());

    let mut proc_info = PROCESS_INFORMATION::default();
    unsafe {
        CreateProcessW(
            PCWSTR(null()),
            Some(commandline),
            None,
            None,
            false,
            EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT,
            None,
            PCWSTR(null()),
            &startup_info.StartupInfo,
            &mut proc_info,
        )?;
    }

    Ok(proc_info)
}

/// Resizes the pseudo console to the specified size.
fn resize_pseudo_console(console: HPCON, size: COORD) -> Result<()> {
    unsafe { windows::Win32::System::Console::ResizePseudoConsole(console, size)? };
    Ok(())
}

/// Closes the pseudo console and releases resources.
fn close_pseudo_console(console: HPCON) -> Result<()> {
    unsafe { windows::Win32::System::Console::ClosePseudoConsole(console) };
    Ok(())
}

fn main() -> Result<()> {
    println!("=== Pseudo Console Demo ===\n");

    let custom_size = COORD { X: 80, Y: 25 };
    println!(
        "Creating pseudo console with size: {}x{}",
        custom_size.X, custom_size.Y
    );

    let (console, con_reader, con_writer) = create_pseudo_console(custom_size)?;
    println!("✓ Pseudo console created successfully");

    let startup_info = initialize_startup_info(&console)?;
    println!("✓ Startup info initialized with pseudo console attribute");

    let command = "cmd.exe";
    println!("\nCreating process: {}", command);
    let proc_info = create_process_attached_to_conpty(command, &startup_info)?;
    println!("✓ Process created with PID: {}", unsafe { GetProcessId(proc_info.hProcess) });

    println!("\nWaiting for process to complete...");
    let wait_result = unsafe { WaitForSingleObject(proc_info.hProcess, INFINITE) };
    if wait_result != WAIT_EVENT(0) {
        return Err(Error::from_thread());
    }
    println!("✓ Process completed");

    let mut exit_code = 0;
    unsafe { GetExitCodeProcess(proc_info.hProcess, &mut exit_code)? };
    println!("Exit code: {}", exit_code);

    let new_size = COORD { X: 120, Y: 30 };
    println!(
        "\nResizing pseudo console to: {}x{}",
        new_size.X, new_size.Y
    );
    resize_pseudo_console(console, new_size)?;
    println!("✓ Pseudo console resized successfully");

    close_pseudo_console(console)?;
    println!("✓ Pseudo console closed");

    unsafe {
        CloseHandle(proc_info.hProcess)?;
        CloseHandle(proc_info.hThread)?;
    }
    println!("✓ Process handles closed");

    unsafe {
        CloseHandle(con_reader)?;
        CloseHandle(con_writer)?;
    }
    println!("✓ Pipe handles closed");

    println!("\n=== Demo Complete ===");
    Ok(())
}
