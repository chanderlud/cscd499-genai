use windows::core::{Error, HRESULT};
use windows::Win32::Globalization::{AdjustCalendarDate, CALDATETIME, CALDATETIME_DATEUNIT};

fn call_adjust_calendar_date() -> HRESULT {
    let mut cal = CALDATETIME::default();
    let unit = CALDATETIME_DATEUNIT(1);
    let amount: i32 = 0;

    // SAFETY: AdjustCalendarDate expects a valid mutable pointer to a CALDATETIME struct.
    // We pass a reference to a properly aligned and initialized struct.
    let success = unsafe { AdjustCalendarDate(&mut cal, unit, amount) };

    if success == false {
        Error::from_thread().code()
    } else {
        HRESULT(0)
    }
}
