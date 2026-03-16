use windows::core::{Error, Result, HRESULT};

pub fn hresult_to_result(hr: HRESULT) -> Result<()> {
    if hr.is_ok() {
        Ok(())
    } else {
        Err(Error::from_hresult(hr))
    }
}