#[macro_use]
extern crate bc;
extern crate libc;

#[macro_use(defer)] extern crate scopeguard;

use std::error::Error;
use std::fmt;
use std::io::Write;

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
        let ptr = &mut saved_termattr;
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
    defer!(
        unsafe {
            libc::tcsetattr(0, libc::TCSANOW, &saved_termattr);
        }
    );
    let mut buf: [libc::c_char; 1] = [0; 1];
    let ptr = &mut buf;

    print!("\r(dntk): ");
    let mut input_vec = Vec::new();
    loop {
        let r = unsafe { libc::read(0, ptr.as_ptr() as *mut libc::c_void, 1) };
        if r > 0 {
            let input_char = ptr[0] as u8;
            match char_scan(input_char) {
                None => (),
                Some(i) => {
                    &input_vec.push(i);
                    },
            }
            let p1 = "\r(dntk): ";
            print!("{}", p1);
            let p2 = std::str::from_utf8(&input_vec).unwrap_or(&"error occured");
            print!("{}", p2);
            let p3 = " = ";
            print!("{}", p3);
            match bc!(format!("{}", p2)) {
                Ok(p4) => {
                    print!("{}", &p4);
                    },
                _ => (),
            }
        }
        std::io::stdout().flush().unwrap();
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

fn char_scan(ascii_char: u8) -> Option<u8> {
    match ascii_char {
        48 => Some(48),
        49 => Some(49),
        50 => Some(50),
        51 => Some(51),
        52 => Some(52),
        53 => Some(53),
        54 => Some(54),
        55 => Some(55),
        56 => Some(56),
        57 => Some(57),
        115 => Some(115),
        99 => Some(99),
        97 => Some(97),
        108 => Some(108),
        101 => Some(101),
        106 => Some(106),
        123 => Some(123),
        125 => Some(125),
        43 => Some(43),
        45 => Some(45),
        42 => Some(42),
        47 => Some(47),
        10 => Some(10), // \n
        27 => Some(27), // escape key
        127 => Some(127), // delete key
        _ => None,
    }
}