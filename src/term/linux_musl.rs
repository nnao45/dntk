fn get_termattr() -> libc::termios {
    libc::termios {
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_cc: [0u8; 32],
        __c_ispeed: 0,
        __c_ospeed: 0,
        c_line: 0,
    }
}