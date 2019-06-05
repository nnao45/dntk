#[cfg(target_os = "windows")]
pub fn enable_ansi() -> Result<(), u32> {
    if std::env::var_os("DNTK_ENV") != Some(std::ffi::OsString::from("TEST")) {
        use winapi::um::processenv::GetStdHandle;
        use winapi::um::errhandlingapi::GetLastError;
        use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
        use winapi::um::handleapi::INVALID_HANDLE_VALUE;

        const STD_OUT_HANDLE: u32 = -11i32 as u32;
        const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;

        unsafe {
            // https://docs.microsoft.com/en-us/windows/console/getstdhandle
            let std_out_handle = GetStdHandle(STD_OUT_HANDLE);
            if std_out_handle == INVALID_HANDLE_VALUE
            {
                return Err(GetLastError()); 
            }
    
            // https://docs.microsoft.com/en-us/windows/console/getconsolemode
            let mut console_mode: u32 = 0;
            if 0 == GetConsoleMode(std_out_handle, &mut console_mode)
            {
                return Err(GetLastError());
            }

            // VT processing not already enabled?
            if console_mode & ENABLE_VIRTUAL_TERMINAL_PROCESSING == 0 {
                // https://docs.microsoft.com/en-us/windows/console/setconsolemode
                if 0 == SetConsoleMode(std_out_handle, console_mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING)
                {
                    return Err(GetLastError()); 
                }
            }
        }
    }

    return Ok(());
}