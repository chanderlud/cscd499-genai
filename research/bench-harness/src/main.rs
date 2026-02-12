use std::path::PathBuf;
use windows::{
    core::*
};

pub fn pick_folder(title: &str) -> Result<Option<PathBuf>> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)?;

        let dialog: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_ALL)?;

        let title = HSTRING::from(title);
        dialog.SetTitle(&title)?;

        let mut options: DWORD = 0;
        dialog.GetOptions(&mut options)?;
        options |= FOS_PICKFOLDERS.0;
        dialog.SetOptions(options)?;

        let result = dialog.Show(None);

        if result == S_OK {
            let result: IShellItem = dialog.GetResult()?;
            let path = result.GetDisplayName(SIGDN_FILESYSPATH)?;
            let path_str = path.to_string_lossy();
            Ok(Some(PathBuf::from(path_str)))
        } else {
            Ok(None)
        }
    }
}

fn main() {

}