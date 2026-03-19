use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::SystemServices::IMAGE_DOS_HEADER;

fn get_instance_handle() -> HMODULE {
    extern "C" {
        static __ImageBase: IMAGE_DOS_HEADER;
    }

    HMODULE(unsafe { &__ImageBase as *const _ as _ })
}

fn main() {
    let instance = get_instance_handle();
    println!("Instance handle: {:?}", instance);
}
