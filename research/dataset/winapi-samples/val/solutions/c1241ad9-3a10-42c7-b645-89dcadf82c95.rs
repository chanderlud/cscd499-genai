use std::thread;
use std::time::Duration;
use windows::core::{Error, Result, HRESULT, PCWSTR};

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

// Define the PDH types and functions manually since the feature isn't enabled
// These are the actual Win32 API signatures
type PdhHquery = isize;
type PdhHcounter = isize;

#[link(name = "pdh")]
extern "system" {
    fn PdhOpenQueryW(szdatasource: PCWSTR, dwuserdata: usize, phquery: *mut PdhHquery) -> u32;

    fn PdhAddCounterW(
        hquery: PdhHquery,
        szfullcounterpath: PCWSTR,
        dwuserdata: usize,
        phcounter: *mut PdhHcounter,
    ) -> u32;

    fn PdhCollectQueryData(hquery: PdhHquery) -> u32;

    fn PdhGetFormattedCounterValue(
        hcounter: PdhHcounter,
        dwformat: u32,
        lpdwtype: *mut u32,
        pvalue: *mut PDH_FMT_COUNTERVALUE,
    ) -> u32;

    fn PdhCloseQuery(hquery: PdhHquery) -> u32;
}

const PDH_FMT_DOUBLE: u32 = 0x00000200;

#[repr(C)]
struct PDH_FMT_COUNTERVALUE {
    anonymous: PDH_FMT_COUNTERVALUE_0,
    #[cfg(target_arch = "x86_64")]
    _padding: u32,
}

#[repr(C)]
union PDH_FMT_COUNTERVALUE_0 {
    longvalue: i32,
    doublevalue: f64,
    largevalue: i64,
    ansi: usize,
    wide: usize,
}

struct ProcessorTimeIterator {
    query: PdhHquery,
    counter: PdhHcounter,
}

impl ProcessorTimeIterator {
    fn new() -> Result<Self> {
        let mut query: PdhHquery = 0;
        let mut counter: PdhHcounter = 0;

        // Open PDH query
        let status = unsafe { PdhOpenQueryW(PCWSTR::null(), 0, &mut query) };
        if status != 0 {
            return Err(Error::from_hresult(HRESULT::from_win32(status)));
        }

        // Add processor time counter for first CPU core
        let counter_path = wide_null("\\Processor(0)\\% Processor Time");
        let status =
            unsafe { PdhAddCounterW(query, PCWSTR(counter_path.as_ptr()), 0, &mut counter) };
        if status != 0 {
            unsafe { PdhCloseQuery(query) };
            return Err(Error::from_hresult(HRESULT::from_win32(status)));
        }

        Ok(Self { query, counter })
    }
}

impl Iterator for ProcessorTimeIterator {
    type Item = Result<f64>;

    fn next(&mut self) -> Option<Self::Item> {
        // Wait for one second between samples
        thread::sleep(Duration::from_secs(1));

        // Collect new data
        let status = unsafe { PdhCollectQueryData(self.query) };
        if status != 0 {
            return Some(Err(Error::from_hresult(HRESULT::from_win32(status))));
        }

        // Get formatted counter value
        let mut value = PDH_FMT_COUNTERVALUE {
            anonymous: PDH_FMT_COUNTERVALUE_0 { doublevalue: 0.0 },
            #[cfg(target_arch = "x86_64")]
            _padding: 0,
        };
        let mut type_ = 0u32;
        let status = unsafe {
            PdhGetFormattedCounterValue(self.counter, PDH_FMT_DOUBLE, &mut type_, &mut value)
        };
        if status != 0 {
            return Some(Err(Error::from_hresult(HRESULT::from_win32(status))));
        }

        Some(Ok(unsafe { value.anonymous.doublevalue }))
    }
}

impl Drop for ProcessorTimeIterator {
    fn drop(&mut self) {
        unsafe {
            PdhCloseQuery(self.query);
        }
    }
}

fn processor_time_iter() -> Result<impl Iterator<Item = Result<f64>>> {
    ProcessorTimeIterator::new()
}

fn main() -> Result<()> {
    let mut iter = processor_time_iter()?;
    for _ in 0..5 {
        match iter.next() {
            Some(Ok(value)) => println!("CPU usage: {:.2}%", value),
            Some(Err(e)) => eprintln!("Error: {}", e),
            None => break,
        }
    }
    Ok(())
}
