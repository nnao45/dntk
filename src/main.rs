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

    //if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
        //let saved_termattr = term::get_termattr();
        //defer!(
        //    unsafe {
        //        libc::tcsetattr(0, libc::TCSANOW, &saved_termattr);
        //    }
        //);
    //}
    
    let dntker = &mut dntker::Dntker::new();
    dntker.run();
}