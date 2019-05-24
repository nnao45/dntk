#[macro_use]
extern crate bc;
extern crate libc;

#[macro_use(defer)]
extern crate scopeguard;

mod util;

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
        let saved_termattr_ptr = &mut saved_termattr;
        libc::tcgetattr(0, saved_termattr_ptr);
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

    print!("{}", util::DNTK_PROMPT);
    let mut input_vec = Vec::new();
    let mut before_printed_len = 0;
    let mut before_printed_result_len = 0;
    loop {
        let r = unsafe { libc::read(0, ptr.as_ptr() as *mut libc::c_void, 1) };
        if r > 0 {
            let input_char = ptr[0] as u8;
            match char_scan(input_char) {
                None => (),
                Some(util::ASCII_CODE_NEWLINE) => {
                    print!("\n");
                    break
                },
                Some(util::ASCII_CODE_ESCAPE) => {
                    print!("\n");
                    break
                },
                Some(util::ASCII_CODE_DELETE) => {
                    &input_vec.pop();
                },
                Some(i) => {
                    &input_vec.push(i);
                    },
            }
            print!("\r{}", (0..before_printed_len).map(|_| " ").collect::<String>());
            let p1 = format!("{}", util::DNTK_PROMPT);
            let p2 = std::str::from_utf8(&input_vec).unwrap_or(&"error occured");
            let p3 = " = ";
            match bc!(format!("{}", p2)) {
                Ok(p4) => {
                    before_printed_result_len = p4.to_string().len();
                    before_printed_len = p1.to_string().len() + p2.to_string().len() + p3.to_string().len() + p4.to_string().len();
                    print!("{}{}{}{}{}{}", util::COLOR_CYAN_HEADER, p1, p2, p3, &p4, util::COLOR_PLAIN_HEADER);
                    print!("{}{}{}", util::CURSOR_MOVE_ES_HEAD, (p3.to_string().len() + &p4.to_string().len()), util::CURSOR_MOVE_ES_BACK);
                    },
                _ => {
                    before_printed_len = p1.to_string().len() + p2.to_string().len() + p3.to_string().len() + before_printed_result_len;
                    print!("{}{}{}{}{}", util::COLOR_MAGENDA_HEADER, p1, p2, p3, util::COLOR_PLAIN_HEADER);
                    print!("{}{}{}", util::CURSOR_MOVE_ES_HEAD, p3.to_string().len(), util::CURSOR_MOVE_ES_BACK);
                },
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
        util::ASCII_CODE_ZERO       => Some(util::ASCII_CODE_ZERO      ), // 0
        util::ASCII_CODE_ONE        => Some(util::ASCII_CODE_ONE       ), // 1
        util::ASCII_CODE_TWO        => Some(util::ASCII_CODE_TWO       ), // 2
        util::ASCII_CODE_THREE      => Some(util::ASCII_CODE_THREE     ), // 3
        util::ASCII_CODE_FOUR       => Some(util::ASCII_CODE_FOUR      ), // 4
        util::ASCII_CODE_FIVE       => Some(util::ASCII_CODE_FIVE      ), // 5
        util::ASCII_CODE_SIX        => Some(util::ASCII_CODE_SIX       ), // 6
        util::ASCII_CODE_SEVEN      => Some(util::ASCII_CODE_SEVEN     ), // 7
        util::ASCII_CODE_EIGHT      => Some(util::ASCII_CODE_EIGHT     ), // 8
        util::ASCII_CODE_NINE       => Some(util::ASCII_CODE_NINE      ), // 9
        util::ASCII_CODE_S          => Some(util::ASCII_CODE_S         ), // s
        util::ASCII_CODE_C          => Some(util::ASCII_CODE_C         ), // c
        util::ASCII_CODE_A          => Some(util::ASCII_CODE_A         ), // a
        util::ASCII_CODE_L          => Some(util::ASCII_CODE_L         ), // l
        util::ASCII_CODE_E          => Some(util::ASCII_CODE_E         ), // e
        util::ASCII_CODE_J          => Some(util::ASCII_CODE_J         ), // j
        util::ASCII_CODE_ROUNDLEFT  => Some(util::ASCII_CODE_ROUNDLEFT ), // (
        util::ASCII_CODE_ROUNDRIGHT => Some(util::ASCII_CODE_ROUNDRIGHT), // )
        util::ASCII_CODE_PLUS       => Some(util::ASCII_CODE_PLUS      ), // +
        util::ASCII_CODE_MINUS      => Some(util::ASCII_CODE_MINUS     ), // -
        util::ASCII_CODE_ASTERISK   => Some(util::ASCII_CODE_ASTERISK  ), // *
        util::ASCII_CODE_SLUSH      => Some(util::ASCII_CODE_SLUSH     ), // /
        util::ASCII_CODE_PERIOD     => Some(util::ASCII_CODE_PERIOD    ), // .
        util::ASCII_CODE_EQUAL      => Some(util::ASCII_CODE_EQUAL     ), // =
        util::ASCII_CODE_SEMICOLON  => Some(util::ASCII_CODE_SEMICOLON ), // ;
        util::ASCII_CODE_NEWLINE    => Some(util::ASCII_CODE_NEWLINE   ), // \n
        util::ASCII_CODE_ESCAPE     => Some(util::ASCII_CODE_ESCAPE    ), // escape key
        util::ASCII_CODE_DELETE     => Some(util::ASCII_CODE_DELETE    ), // delete key
        util::ASCII_CODE_SPACE      => Some(util::ASCII_CODE_SPACE     ), // white space key
        _ => None,
    }
}