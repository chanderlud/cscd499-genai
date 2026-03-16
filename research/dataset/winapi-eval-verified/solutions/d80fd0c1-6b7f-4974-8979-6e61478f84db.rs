use std::ffi::c_void;
use std::io::{self, Error, ErrorKind};
use std::mem::size_of;
use std::thread;
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, HANDLE, INVALID_HANDLE_VALUE, WAIT_OBJECT_0, WAIT_TIMEOUT,
};
use windows::Win32::Storage::FileSystem::ReadFile;
use windows::Win32::System::Console::{COORD, ClosePseudoConsole, CreatePseudoConsole, HPCON};
use windows::Win32::System::Pipes::CreatePipe;
use windows::Win32::System::Threading::{
    CreateProcessW, DeleteProcThreadAttributeList, EXTENDED_STARTUPINFO_PRESENT,
    InitializeProcThreadAttributeList, LPPROC_THREAD_ATTRIBUTE_LIST,
    PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE, PROCESS_INFORMATION, STARTUPINFOEXW, TerminateProcess,
    UpdateProcThreadAttribute, WaitForSingleObject,
};
use windows::core::{PCWSTR, PWSTR};

fn win32_err(context: &'static str, err: impl std::fmt::Display) -> io::Error {
    Error::other(format!("{context} failed: {err}"))
}

struct OwnedHandle(HANDLE);

impl OwnedHandle {
    fn null() -> Self {
        Self(HANDLE::default())
    }

    fn is_valid(&self) -> bool {
        self.0 != HANDLE::default() && self.0 != INVALID_HANDLE_VALUE
    }

    fn raw(&self) -> HANDLE {
        self.0
    }

    fn take(&mut self) -> HANDLE {
        let h = self.0;
        self.0 = HANDLE::default();
        h
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        if self.is_valid() {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
}

struct OwnedPseudoConsole(HPCON);

impl OwnedPseudoConsole {
    fn null() -> Self {
        Self(HPCON::default())
    }

    fn raw(&self) -> HPCON {
        self.0
    }

    fn close(&mut self) {
        if self.0 != HPCON::default() {
            unsafe {
                ClosePseudoConsole(self.0);
            }
            self.0 = HPCON::default();
        }
    }
}

impl Drop for OwnedPseudoConsole {
    fn drop(&mut self) {
        self.close();
    }
}

struct AttributeList {
    buf: Vec<u8>,
    ptr: LPPROC_THREAD_ATTRIBUTE_LIST,
}

impl AttributeList {
    fn new() -> io::Result<Self> {
        let mut bytes = 0usize;

        unsafe {
            let _ = InitializeProcThreadAttributeList(None, 1, Some(0), &mut bytes);
        }

        if bytes == 0 {
            return Err(Error::last_os_error());
        }

        let mut buf = vec![0u8; bytes];
        let ptr = LPPROC_THREAD_ATTRIBUTE_LIST(buf.as_mut_ptr().cast());

        unsafe {
            InitializeProcThreadAttributeList(Some(ptr), 1, Some(0), &mut bytes)
                .map_err(|e| win32_err("InitializeProcThreadAttributeList", e))?;
        }

        Ok(Self { buf, ptr })
    }

    fn as_ptr(&mut self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        self.ptr
    }
}

impl Drop for AttributeList {
    fn drop(&mut self) {
        if !self.ptr.0.is_null() {
            unsafe {
                DeleteProcThreadAttributeList(self.ptr);
            }
        }
        let _ = &self.buf;
    }
}

fn to_wide_mut(s: &str) -> Vec<u16> {
    let mut v: Vec<u16> = s.encode_utf16().collect();
    v.push(0);
    v
}

fn spawn_reader_thread(output_read: HANDLE) -> thread::JoinHandle<Vec<u8>> {
    let raw = output_read.0 as usize;

    thread::spawn(move || {
        let output_read = HANDLE(raw as *mut c_void);

        let mut out = Vec::new();
        let mut buf = [0u8; 4096];

        loop {
            let mut read = 0u32;

            let result =
                unsafe { ReadFile(output_read, Some(&mut buf[..]), Some(&mut read), None) };

            match result {
                Ok(()) if read > 0 => {
                    out.extend_from_slice(&buf[..read as usize]);
                }
                _ => break,
            }
        }

        unsafe {
            let _ = CloseHandle(output_read);
        }

        out
    })
}

pub fn run_in_conpty(command_line: &str, timeout_ms: u32) -> io::Result<Vec<u8>> {
    if command_line.trim().is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "command_line is empty"));
    }

    unsafe {
        let mut pty_input_read = OwnedHandle::null();
        let mut host_input_write = OwnedHandle::null();
        let mut host_output_read = OwnedHandle::null();
        let mut pty_output_write = OwnedHandle::null();

        CreatePipe(&mut pty_input_read.0, &mut host_input_write.0, None, 0)
            .map_err(|e| win32_err("CreatePipe(stdin)", e))?;

        CreatePipe(&mut host_output_read.0, &mut pty_output_write.0, None, 0)
            .map_err(|e| win32_err("CreatePipe(stdout)", e))?;

        let mut pty = OwnedPseudoConsole::null();
        pty.0 = CreatePseudoConsole(
            COORD { X: 80, Y: 25 },
            pty_input_read.raw(),
            pty_output_write.raw(),
            0,
        )
            .map_err(|e| win32_err("CreatePseudoConsole", e))?;

        let mut attr_list = AttributeList::new()?;
        let mut si = STARTUPINFOEXW::default();
        si.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
        si.lpAttributeList = attr_list.as_ptr();

        let hpc = pty.raw();
        UpdateProcThreadAttribute(
            si.lpAttributeList,
            0,
            PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE as usize,
            Some(hpc.0 as *const c_void),
            size_of::<HPCON>(),
            None,
            None,
        )
            .map_err(|e| win32_err("UpdateProcThreadAttribute", e))?;

        let mut cmd = to_wide_mut(command_line);
        let mut pi: PROCESS_INFORMATION = std::mem::zeroed();

        CreateProcessW(
            PCWSTR::null(),
            Some(PWSTR(cmd.as_mut_ptr())),
            None,
            None,
            false,
            EXTENDED_STARTUPINFO_PRESENT,
            None,
            PCWSTR::null(),
            &si.StartupInfo,
            &mut pi,
        )
            .map_err(|e| win32_err("CreateProcessW", e))?;

        let process = OwnedHandle(pi.hProcess);
        let _thread = OwnedHandle(pi.hThread);

        // These ends are owned by the pseudoconsole now.
        drop(pty_input_read);
        drop(pty_output_write);

        let reader = spawn_reader_thread(host_output_read.take());

        // IMPORTANT:
        // Keep host_input_write open for the lifetime of the session.
        // Do not signal EOF immediately.

        let wait = WaitForSingleObject(process.raw(), timeout_ms);

        match wait {
            WAIT_OBJECT_0 => {
                pty.close();

                let raw = reader
                    .join()
                    .map_err(|_| Error::other("output reader thread panicked"))?;

                drop(host_input_write);

                Ok(raw)
            }
            WAIT_TIMEOUT => {
                let _ = TerminateProcess(process.raw(), 1);

                // Now close the input side too.
                drop(host_input_write);

                pty.close();

                let _ = reader.join();
                Err(Error::new(ErrorKind::TimedOut, "process timed out"))
            }
            _ => {
                let err = Error::from_raw_os_error(GetLastError().0 as i32);

                let _ = TerminateProcess(process.raw(), 1);
                drop(host_input_write);
                pty.close();
                let _ = reader.join();

                Err(err)
            }
        }
    }
}
