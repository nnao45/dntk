extern crate libc;
extern crate clap;

#[macro_use(defer)]
extern crate scopeguard;

#[cfg(target_os = "windows")]
extern crate winconsole;

#[cfg(target_os = "windows")]
extern crate ansi_term;

mod term;
mod dntker;
mod meta;

fn main() {
    let _matches = meta::build_cli().get_matches();

    #[cfg(not(target_os = "windows"))]
    let saved_termattr = term::get_termattr();
    defer!(
        unsafe {
            #[cfg(not(target_os = "windows"))]
            libc::tcsetattr(0, libc::TCSANOW, &saved_termattr);
        }
    );

    #[cfg(target_os = "windows")]
    ansi_term::enable_ansi_support().unwrap();

    let dntker = &mut dntker::Dntker::new();
    dntker.run();
}