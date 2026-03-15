use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::SystemServices::IMAGE_DOS_HEADER;

fn get_instance_handle() -> HMODULE {
    // Gets the instance handle by taking the address of the
    // pseudo-variable created by the microsoft linker:
    // https://devblogs.microsoft.com/oldnewthing/20041025-00/?p=37483

    // This is preferred over GetModuleHandle(NULL) because it also works in DLLs:
    // https://stackoverflow.com/questions/21718027/getmodulehandlenull-vs-hinstance

    extern "C" {
        static __ImageBase: IMAGE_DOS_HEADER;
    }

    HMODULE(unsafe { &__ImageBase as *const _ as _ })
}

fn main() {
    let instance = get_instance_handle();
    println!("Instance handle: {:?}", instance);
}
