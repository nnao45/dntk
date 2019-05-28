extern crate libc;
extern crate clap;
extern crate wincolor;

#[macro_use(defer)]
extern crate scopeguard;

#[cfg(target_os = "windows")]
extern crate winconsole;

mod term;
mod dntker;
mod meta;

fn main() {
    let _matches = meta::build_cli().get_matches();

    #[cfg(target_os = "macos")]
    let saved_termattr = term::get_termattr();
    #[cfg(target_os = "linux")]
    let saved_termattr = term::get_termattr();
    defer!(
        unsafe {
            #[cfg(target_os = "macos")]
            libc::tcsetattr(0, libc::TCSANOW, &saved_termattr);
            #[cfg(target_os = "linux")]
            libc::tcsetattr(0, libc::TCSANOW, &saved_termattr);
        }
    );

    let dntker = &mut dntker::Dntker::new();
    dntker.run();
}