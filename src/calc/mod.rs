mod util;

use std::io::Write;

use bc::bc;

pub struct Dntker {
    buffer: [libc::c_char; 1],
    input_vec: Vec<u8>,
    before_printed_len: usize,
    before_printed_result_len: usize,
    before_printed_statement_len: usize,
    currnet_cur_pos: usize,
}

impl Dntker {
    pub fn new() -> Self {
        Dntker {
            buffer: [0; 1],
            input_vec: Vec::new(),
            before_printed_len: 0,
            before_printed_result_len: 0,
            before_printed_statement_len: 0,
            currnet_cur_pos: 0,
        }
    }

    pub fn run(mut self) {
        let ptr = &mut self.buffer;

        print!("{}", util::DNTK_PROMPT);
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
                        if self.currnet_cur_pos > 0 {
                            self.currnet_cur_pos -= 1;
                            &self.input_vec.remove(self.currnet_cur_pos);
                        }
                    },
                    Some(util::ASCII_CODE_SQUARELEFT) => {
                        if self.currnet_cur_pos > 0 {
                            self.currnet_cur_pos -= 1;
                        }
                    },
                    Some(util::ASCII_CODE_SQUARERIGHT) => {
                        if self.currnet_cur_pos < self.before_printed_statement_len {
                            self.currnet_cur_pos += 1;
                        }
                    },
                    Some(i) => {
                        self.currnet_cur_pos += 1;
                        &self.input_vec.insert(self.currnet_cur_pos-1, i);
                        },
                }
                print!("\r{}", (0..self.before_printed_len).map(|_| " ").collect::<String>());
                let p1 = format!("{}", util::DNTK_PROMPT);
                let p2 = std::str::from_utf8(&self.input_vec).unwrap_or(&"error occured");
                let p3 = " = ";
                match bc!(format!("{}", p2)) {
                    Ok(p4) => {
                        self.before_printed_result_len = p4.to_string().len();
                        self.before_printed_statement_len = p2.to_string().len();
                        self.before_printed_len = p1.to_string().len() + self.before_printed_statement_len + p3.to_string().len() + self.before_printed_result_len;
                        let pos_differnce = self.before_printed_statement_len - self.currnet_cur_pos;
                        print!("{}{}{}{}{}{}", util::COLOR_CYAN_HEADER, p1, p2, p3, &p4, util::COLOR_PLAIN_HEADER);
                        print!("{}{}{}", util::CURSOR_MOVE_ES_HEAD, (p3.to_string().len() + self.before_printed_result_len + pos_differnce), util::CURSOR_MOVE_ES_BACK);
                        },
                    _ => {
                        self.before_printed_statement_len = p2.to_string().len();
                        self.before_printed_len = p1.to_string().len() +  self.before_printed_statement_len + p3.to_string().len() + self.before_printed_result_len;
                        let pos_differnce =  self.before_printed_statement_len - self.currnet_cur_pos;
                        print!("{}{}{}{}{}", util::COLOR_MAGENDA_HEADER, p1, p2, p3, util::COLOR_PLAIN_HEADER);
                        print!("{}{}{}", util::CURSOR_MOVE_ES_HEAD, (p3.to_string().len() + pos_differnce), util::CURSOR_MOVE_ES_BACK);
                    },
                }
            }
            std::io::stdout().flush().unwrap();
        }
    }
}


fn char_scan(ascii_char: u8) -> Option<u8> {
    match ascii_char {
        util::ASCII_CODE_ZERO        => Some(util::ASCII_CODE_ZERO       ), // 0
        util::ASCII_CODE_ONE         => Some(util::ASCII_CODE_ONE        ), // 1
        util::ASCII_CODE_TWO         => Some(util::ASCII_CODE_TWO        ), // 2
        util::ASCII_CODE_THREE       => Some(util::ASCII_CODE_THREE      ), // 3
        util::ASCII_CODE_FOUR        => Some(util::ASCII_CODE_FOUR       ), // 4
        util::ASCII_CODE_FIVE        => Some(util::ASCII_CODE_FIVE       ), // 5
        util::ASCII_CODE_SIX         => Some(util::ASCII_CODE_SIX        ), // 6
        util::ASCII_CODE_SEVEN       => Some(util::ASCII_CODE_SEVEN      ), // 7
        util::ASCII_CODE_EIGHT       => Some(util::ASCII_CODE_EIGHT      ), // 8
        util::ASCII_CODE_NINE        => Some(util::ASCII_CODE_NINE       ), // 9
        util::ASCII_CODE_S           => Some(util::ASCII_CODE_S          ), // s
        util::ASCII_CODE_C           => Some(util::ASCII_CODE_C          ), // c
        util::ASCII_CODE_A           => Some(util::ASCII_CODE_A          ), // a
        util::ASCII_CODE_L           => Some(util::ASCII_CODE_L          ), // l
        util::ASCII_CODE_E           => Some(util::ASCII_CODE_E          ), // e
        util::ASCII_CODE_J           => Some(util::ASCII_CODE_J          ), // j
        util::ASCII_CODE_ROUNDLEFT   => Some(util::ASCII_CODE_ROUNDLEFT  ), // (
        util::ASCII_CODE_ROUNDRIGHT  => Some(util::ASCII_CODE_ROUNDRIGHT ), // )
        util::ASCII_CODE_SQUARELEFT  => Some(util::ASCII_CODE_SQUARELEFT ), // [
        util::ASCII_CODE_SQUARERIGHT => Some(util::ASCII_CODE_SQUARERIGHT), // ]
        util::ASCII_CODE_PLUS        => Some(util::ASCII_CODE_PLUS       ), // +
        util::ASCII_CODE_MINUS       => Some(util::ASCII_CODE_MINUS      ), // -
        util::ASCII_CODE_ASTERISK    => Some(util::ASCII_CODE_ASTERISK   ), // *
        util::ASCII_CODE_SLUSH       => Some(util::ASCII_CODE_SLUSH      ), // /
        util::ASCII_CODE_PERIOD      => Some(util::ASCII_CODE_PERIOD     ), // .
        util::ASCII_CODE_EQUAL       => Some(util::ASCII_CODE_EQUAL      ), // =
        util::ASCII_CODE_SEMICOLON   => Some(util::ASCII_CODE_SEMICOLON  ), // ;
        util::ASCII_CODE_NEWLINE     => Some(util::ASCII_CODE_NEWLINE    ), // \n
        util::ASCII_CODE_ESCAPE      => Some(util::ASCII_CODE_ESCAPE     ), // escape key
        util::ASCII_CODE_DELETE      => Some(util::ASCII_CODE_DELETE     ), // delete key
        util::ASCII_CODE_SPACE       => Some(util::ASCII_CODE_SPACE      ), // white space key
        _ => None,
    }
}