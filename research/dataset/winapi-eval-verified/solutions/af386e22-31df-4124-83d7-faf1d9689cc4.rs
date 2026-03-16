use std::ffi::c_void;
use windows::Win32::System::Threading::{
    ConvertFiberToThread, ConvertThreadToFiber, CreateFiber, DeleteFiber, SwitchToFiber,
};
use windows::core::{Error, Result};

#[derive(Debug)]
struct FiberContext {
    id: u32,
    remaining: u32,
    scheduler_fiber: *mut c_void,
    log: *mut Vec<u32>,
}

unsafe extern "system" fn fiber_proc(param: *mut c_void) {
    let ctx = unsafe { &mut *(param as *mut FiberContext) };

    while ctx.remaining > 0 {
        unsafe {
            (*ctx.log).push(ctx.id);
        }
        ctx.remaining -= 1;

        unsafe {
            SwitchToFiber(ctx.scheduler_fiber);
        }
    }

    // Never return from the fiber procedure. Returning would terminate the thread.
    unsafe {
        SwitchToFiber(ctx.scheduler_fiber);
    }
}

pub fn fiber_round_robin(n_fibers: u32, iters: u32) -> Result<Vec<u32>> {
    if n_fibers == 0 || iters == 0 {
        return Ok(Vec::new());
    }

    let mut log = Vec::with_capacity((n_fibers as usize) * (iters as usize));
    let log_ptr: *mut Vec<u32> = &mut log;

    unsafe {
        let scheduler_fiber = ConvertThreadToFiber(None);
        if scheduler_fiber.is_null() {
            return Err(Error::from_thread());
        }

        let run_result = (|| -> Result<()> {
            let mut contexts = Vec::with_capacity(n_fibers as usize);
            for id in 0..n_fibers {
                contexts.push(FiberContext {
                    id,
                    remaining: iters,
                    scheduler_fiber,
                    log: log_ptr,
                });
            }

            let mut fibers = Vec::with_capacity(n_fibers as usize);

            for ctx in &mut contexts {
                let fiber = CreateFiber(
                    0,
                    Some(fiber_proc),
                    Some(ctx as *mut FiberContext as *const c_void),
                );

                if fiber.is_null() {
                    for fiber in fibers.drain(..) {
                        DeleteFiber(fiber);
                    }
                    return Err(Error::from_thread());
                }

                fibers.push(fiber);
            }

            for _ in 0..iters {
                for &fiber in &fibers {
                    SwitchToFiber(fiber);
                }
            }

            for fiber in fibers {
                DeleteFiber(fiber);
            }

            Ok(())
        })();

        let convert_back_result = ConvertFiberToThread();

        match (run_result, convert_back_result) {
            (Err(e), _) => return Err(e),
            (Ok(()), Err(e)) => return Err(e),
            (Ok(()), Ok(())) => {}
        }
    }

    Ok(log)
}
