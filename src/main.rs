extern crate libc;
extern crate clap;
extern crate winapi;
extern crate ansi_term;

#[macro_use(defer)]
#[allow(unused_imports)]
extern crate scopeguard;

#[cfg(target_os = "windows")]
extern crate winconsole;

#[macro_use(lazy_static)]
extern crate lazy_static;

mod term;
mod dntker;

fn main() {
    #[cfg(not(target_os = "windows"))]
    let saved_termattr = term::get_termattr();
    #[cfg(not(target_os = "windows"))]
    defer!(
        unsafe {
            libc::tcsetattr(0, libc::TCSANOW, &saved_termattr);
        }
    );

    #[cfg(target_os = "windows")] 
    term::enable_ansi().unwrap();

    let dntker = &mut dntker::Dntker::new();
    dntker.run();
}