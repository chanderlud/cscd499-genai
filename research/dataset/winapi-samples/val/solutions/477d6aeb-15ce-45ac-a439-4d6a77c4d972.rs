use windows::core::{Error, Result};
use windows::Win32::Foundation::{E_INVALIDARG, HWND};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};
use windows::Win32::UI::Shell::{ITaskbarList3, TBPF_NORMAL};

fn set_progress_bar_fraction(hwnd: HWND, numerator: u64, denominator: u64) -> Result<()> {
    // Validate denominator
    if denominator == 0 {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    // Calculate percentage (0-100) from fraction, clamped to [0, 1]
    let percentage = if numerator >= denominator {
        100u32
    } else {
        // Use u128 to avoid overflow during multiplication
        let num = numerator as u128;
        let den = denominator as u128;
        ((num * 100) / den) as u32
    };

    // Create and initialize taskbar list
    let taskbar: ITaskbarList3 = unsafe {
        // SAFETY: CoCreateInstance creates COM objects; caller must have initialized COM
        // CLSID for TaskbarList: {56FDF344-FD6D-11d0-958A-006097C9A090}
        let clsid = windows::core::GUID::from_u128(0x56fdf344_fd6d_11d0_958a_006097c9a090);
        CoCreateInstance(&clsid, None, CLSCTX_ALL)?
    };

    // Initialize the taskbar list interface
    unsafe {
        // SAFETY: HrInit is a valid method of ITaskbarList3
        taskbar.HrInit()?;
    }

    // Set progress state to normal
    unsafe {
        // SAFETY: SetProgressState is a valid method, hwnd is provided by caller
        taskbar.SetProgressState(hwnd, TBPF_NORMAL)?;
    }

    // Set progress value (current, total)
    unsafe {
        // SAFETY: SetProgressValue is a valid method, hwnd is provided by caller
        taskbar.SetProgressValue(hwnd, percentage as u64, 100)?;
    }

    Ok(())
}
