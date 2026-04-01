use windows::core::IUnknown;
use windows::core::GUID;
use windows::core::{Result, HRESULT};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};

fn call_co_create_instance() -> HRESULT {
    let clsid = GUID {
        data1: 0x000214E2,
        data2: 0x0000,
        data3: 0x0000,
        data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    };

    unsafe {
        CoCreateInstance::<_, IUnknown>(&clsid, Option::<&IUnknown>::None, CLSCTX_INPROC_SERVER)
            .into()
    }
}
