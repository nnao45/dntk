extern crate libc;
extern crate clap;

#[macro_use(defer)]
extern crate scopeguard;

mod term;
mod dntker;
mod meta;

fn main() {
    let _matches = meta::build_cli().get_matches();
    let saved_termattr = term::setup();
    defer!(
        unsafe {
            libc::tcsetattr(0, libc::TCSANOW, &saved_termattr);
        }
    );
    let dntker = &mut dntker::Dntker::new();
    dntker.run();
}