#[cfg(target_os = "macos")]
include!("osx.rs");

#[cfg(target_os = "linux")]
include!(env!("LINUX_TERM_LIB"));

pub fn setup() -> libc::termios {
    get_termattr()
}