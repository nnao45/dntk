#[macro_use]
extern crate bc;
extern crate libc;
extern crate sdl2;

use sdl2::keyboard::Keycode;
use sdl2::sys;
use std::convert::From;
use std::error;
use std::error::Error;
use std::ffi::CStr;
use std::fmt;
use std::io::Write;
use std::time::SystemTime;
use std::{thread, time};

fn main() {
    let mut saved_termattr = libc::termios {
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_cc: [0u8; 20],
        c_ispeed: 0,
        c_ospeed: 0,
    };
    unsafe {
        let mut ptr = &mut saved_termattr;
        libc::tcgetattr(0, ptr);
    }
    let mut termattr = saved_termattr;
    termattr.c_lflag = termattr.c_lflag & !(libc::ICANON | libc::ECHO);
    termattr.c_cc[libc::VMIN] = 1;
    termattr.c_cc[libc::VTIME] = 0;
    unsafe {
        libc::tcsetattr(0, libc::TCSANOW, &termattr);
    }
    unsafe {
        libc::fcntl(0, libc::F_SETFL, libc::O_NONBLOCK);
    }
    let mut buf: [libc::c_char; 1] = [0; 1];
    let ptr = &mut buf;

    print!("\r(dntk): ");
    let mut input_vec = Vec::new();
    loop {
        let r = unsafe { libc::read(0, ptr.as_ptr() as *mut libc::c_void, 1) };
        if r > 0 {
            let input_char = ptr[0] as u8 as char;
            // match dntk_scan(input_char) {
            //     None => (),
            //     Some(c) => input_vec.push(c),
            // }
            input_vec.push(input_char);
            print!("\r(dntk): ");
            print!("{:?}", input_vec);
            print!(" = ");
            // print!(
            //     "{}",
            //     bc!(format!(
            //         "{} + {} + {}",
            //         input_vec[0], input_vec[0], input_vec[0]
            //     ))
            //     .unwrap()
            // );
        }
        std::io::stdout().flush().unwrap();
    }
    unsafe {
        libc::tcsetattr(0, libc::TCSANOW, &saved_termattr);
    }
}

#[derive(Debug)]
pub enum ScanError {
    UnkownError,
    UnSupportedError,
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ScanError::UnkownError => f.write_str("Scan key code is unknown"),
            ScanError::UnSupportedError => f.write_str("Scan key code is unsupported"),
        }
    }
}

impl Error for ScanError {
    fn description(&self) -> &str {
        match *self {
            ScanError::UnkownError => "Scan key code is unknown",
            ScanError::UnSupportedError => "Scan key code is unsupported",
        }
    }
}

fn dntk_scan(ascii_char: char) -> Option<char> {
    match ascii_char {
        's' => Some('s'),
        'c' => Some('c'),
        'a' => Some('a'),
        'l' => Some('l'),
        'e' => Some('e'),
        'j' => Some('j'),
        '(' => Some('('),
        ')' => Some(')'),
        '+' => Some('+'),
        '-' => Some('-'),
        '/' => Some('/'),
        '*' => Some('*'),
        '\n' => Some('\n'), // \n
        '\u{1b}' => Some('\u{1b}'), // escape key
        '\u{1b}' => Some('\u{7f}'), // delete key
        _ => None,
    }
}