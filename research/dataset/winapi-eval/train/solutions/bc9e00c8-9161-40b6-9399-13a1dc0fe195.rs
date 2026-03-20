use std::{
    ffi::c_void,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
};

use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE, WAIT_EVENT, WAIT_OBJECT_0, WAIT_TIMEOUT},
        System::Threading::{CreateEventW, INFINITE, SetEvent, WaitForSingleObject},
    },
    core::{Error, HRESULT, PCWSTR, Result},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlarmReport {
    pub initial_probe_timed_out: bool,
    pub awakened_workers: usize,
    pub post_signal_probe_passed: bool,
}

struct OwnedEvent {
    handle: HANDLE,
}

impl OwnedEvent {
    fn new_manual_reset_nonsignaled() -> Result<Self> {
        let handle = unsafe { CreateEventW(None, true, false, PCWSTR::null()) }?;
        Ok(Self { handle })
    }

    fn handle(&self) -> HANDLE {
        self.handle
    }

    fn raw_usize(&self) -> usize {
        self.handle.0 as usize
    }

    fn set(&self) -> Result<()> {
        unsafe { SetEvent(self.handle) }
    }
}

impl Drop for OwnedEvent {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.handle) };
    }
}

fn wait_signaled(handle: HANDLE, timeout_ms: u32) -> Result<bool> {
    let status: WAIT_EVENT = unsafe { WaitForSingleObject(handle, timeout_ms) };

    if status == WAIT_OBJECT_0 {
        Ok(true)
    } else if status == WAIT_TIMEOUT {
        Ok(false)
    } else {
        Err(Error::new(
            HRESULT(0x8000_4005u32 as i32),
            format!("unexpected wait status: {}", status.0),
        ))
    }
}

pub fn run_stage_red_light_rehearsal(worker_count: usize) -> Result<AlarmReport> {
    let event = OwnedEvent::new_manual_reset_nonsignaled()?;

    let initial_probe_timed_out = !wait_signaled(event.handle(), 0)?;

    let awakened_workers = Arc::new(AtomicUsize::new(0));
    let (ready_tx, ready_rx) = mpsc::channel();
    let mut joins = Vec::with_capacity(worker_count);

    for _ in 0..worker_count {
        let ready_tx = ready_tx.clone();
        let awakened_workers = Arc::clone(&awakened_workers);
        let raw_handle = event.raw_usize();

        joins.push(thread::spawn(move || -> Result<()> {
            ready_tx
                .send(())
                .expect("main thread stopped receiving readiness notifications");

            let signaled = wait_signaled(HANDLE(raw_handle as *mut c_void), INFINITE)?;
            if !signaled {
                return Err(Error::new(
                    HRESULT(0x8000_4005u32 as i32),
                    "an infinite wait unexpectedly timed out",
                ));
            }

            awakened_workers.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }));
    }

    drop(ready_tx);

    for _ in 0..worker_count {
        ready_rx
            .recv()
            .expect("a worker failed to report readiness");
    }

    event.set()?;

    for join in joins {
        join.join().expect("worker thread panicked")?;
    }

    let post_signal_probe_passed = wait_signaled(event.handle(), 0)?;
    let awakened_workers = awakened_workers.load(Ordering::SeqCst);

    Ok(AlarmReport {
        initial_probe_timed_out,
        awakened_workers,
        post_signal_probe_passed,
    })
}
