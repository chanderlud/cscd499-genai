use windows::core::{Error, Result};
use windows::Win32::Globalization::{AdjustCalendarDate, CALDATETIME, CALDATETIME_DATEUNIT};

fn call_adjust_calendar_date() -> Result<windows::core::BOOL> {
    let mut cal: CALDATETIME = unsafe { std::mem::zeroed() };
    let unit = CALDATETIME_DATEUNIT(1);
    let amount = 1i32;

    // SAFETY: AdjustCalendarDate requires a valid mutable pointer to a CALDATETIME struct.
    // We provide a zero-initialized struct and check the BOOL return value for failure.
    let result = unsafe { AdjustCalendarDate(&mut cal, unit, amount) };
    if result.0 != 0 {
        Ok(result)
    } else {
        Err(Error::from_thread())
    }
}
