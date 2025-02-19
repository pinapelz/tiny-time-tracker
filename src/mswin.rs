use rfd::FileDialog;
use rfd::MessageDialogResult;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use winapi::um::fileapi::QueryDosDeviceW;
use winapi::shared::minwindef::DWORD;
use rfd::MessageDialog;
use rfd::MessageButtons;


pub fn filechooser_select_executable() -> String {
    let path = FileDialog::new()
    .add_filter("Executable", &["exe"])
    .add_filter("All Files", &["*"])
    .pick_file();
    path.unwrap().to_str().unwrap().to_string()
}


pub fn get_device_path(input: &str) -> Result<String, String> {
    if input.len() < 2 || input.chars().nth(1) != Some(':') {
        return Err("Path must start with a drive letter, e.g. \"C:\"".into());
    }

    let drive_letter = &input[..2];
    let drive_wide: Vec<u16> = OsStr::new(drive_letter)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut buffer: Vec<u16> = vec![0; 1024];
    let buffer_size: DWORD = buffer.len() as DWORD;
    
    let result = unsafe {
        QueryDosDeviceW(drive_wide.as_ptr(), buffer.as_mut_ptr(), buffer_size)
    };

    if result == 0 {
        return Err(format!("Failed to query device for drive {}.", drive_letter));
    }
    
    let device_path_wide = &buffer[..result as usize];
    let first_null = device_path_wide
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(device_path_wide.len());
    
    let device_path = String::from_utf16(&device_path_wide[..first_null])
        .map_err(|e| format!("UTF-16 conversion error: {}", e))?;

    let rest_of_path = if input.len() > 2
        && (input.chars().nth(2) == Some('\\') || input.chars().nth(2) == Some('/'))
    {
        &input[2..]
    } else {
        ""
    };
    let final_path = format!("{}{}", device_path, rest_of_path);
    Ok(final_path)
}

pub fn show_confirmation_dialog(title: &str, message: &str) -> bool {
    let result: rfd::MessageDialogResult = MessageDialog::new()
        .set_title(title)
        .set_description(message)
        .set_buttons(MessageButtons::OkCancel)
        .show();
    result == MessageDialogResult::Yes
}
