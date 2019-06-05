#[cfg(target_os = "macos")]
include!("osx.rs");

#[cfg(target_os = "linux")]
include!(env!("LINUX_TERM_LIB"));

#[cfg(target_os = "windows")]
include!("windows.rs");