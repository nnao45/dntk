extern crate bc;
extern crate libc;

#[macro_use(defer)]
extern crate scopeguard;

mod term;
mod calc;

fn main() {
    let saved_termattr = term::setup();
    defer!(
        unsafe {
            libc::tcsetattr(0, libc::TCSANOW, &saved_termattr);
        }
    );
    let dntker = calc::Dntker::new();
    dntker.run();
}