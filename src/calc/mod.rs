mod util;

use std::io::Write;

use bc::bc;

#[derive(Debug)]
pub struct Dntker {
    input_vec: Vec<u8>,
    before_printed_len: usize,
    before_printed_result_len: usize,
    before_printed_statement_len: usize,
    currnet_cur_pos: usize,
}

#[derive(Debug)]
enum ScanResult {
    BcCode(u8),
    EndCode,
    DeleteCode,
    CurLeftCode,
    CurRightCode,
    UnknownCode,
}

impl Dntker {
    pub fn new() -> Self {
        Dntker {
            input_vec: Vec::new(),
            before_printed_len: 0,
            before_printed_result_len: 0,
            before_printed_statement_len: 0,
            currnet_cur_pos: 0,
        }
    }

    fn fill_whitespace(&self) {
        print!("\r{}", (0..self.before_printed_len).map(|_| " ").collect::<String>());
    }

    fn char_scan(&self, ascii_char: u8) -> ScanResult {
        match ascii_char {
            util::ASCII_CODE_ZERO        => ScanResult::BcCode(util::ASCII_CODE_ZERO                ), // 0
            util::ASCII_CODE_ONE         => ScanResult::BcCode(util::ASCII_CODE_ONE                 ), // 1
            util::ASCII_CODE_TWO         => ScanResult::BcCode(util::ASCII_CODE_TWO                 ), // 2
            util::ASCII_CODE_THREE       => ScanResult::BcCode(util::ASCII_CODE_THREE               ), // 3
            util::ASCII_CODE_FOUR        => ScanResult::BcCode(util::ASCII_CODE_FOUR                ), // 4
            util::ASCII_CODE_FIVE        => ScanResult::BcCode(util::ASCII_CODE_FIVE                ), // 5
            util::ASCII_CODE_SIX         => ScanResult::BcCode(util::ASCII_CODE_SIX                 ), // 6
            util::ASCII_CODE_SEVEN       => ScanResult::BcCode(util::ASCII_CODE_SEVEN               ), // 7
            util::ASCII_CODE_EIGHT       => ScanResult::BcCode(util::ASCII_CODE_EIGHT               ), // 8
            util::ASCII_CODE_NINE        => ScanResult::BcCode(util::ASCII_CODE_NINE                ), // 9
            util::ASCII_CODE_S           => ScanResult::BcCode(util::ASCII_CODE_S                   ), // s
            util::ASCII_CODE_C           => ScanResult::BcCode(util::ASCII_CODE_C                   ), // c
            util::ASCII_CODE_A           => ScanResult::BcCode(util::ASCII_CODE_A                   ), // a
            util::ASCII_CODE_L           => ScanResult::BcCode(util::ASCII_CODE_L                   ), // l
            util::ASCII_CODE_E           => ScanResult::BcCode(util::ASCII_CODE_E                   ), // e
            util::ASCII_CODE_J           => ScanResult::BcCode(util::ASCII_CODE_J                   ), // j
            util::ASCII_CODE_ROUNDLEFT   => ScanResult::BcCode(util::ASCII_CODE_ROUNDLEFT           ), // (
            util::ASCII_CODE_ROUNDRIGHT  => ScanResult::BcCode(util::ASCII_CODE_ROUNDRIGHT          ), // )
            util::ASCII_CODE_SQUARELEFT  => ScanResult::CurLeftCode,                                   // [
            util::ASCII_CODE_SQUARERIGHT => ScanResult::CurRightCode,                                  // ]
            util::ASCII_CODE_PLUS        => ScanResult::BcCode(util::ASCII_CODE_PLUS                ), // +
            util::ASCII_CODE_MINUS       => ScanResult::BcCode(util::ASCII_CODE_MINUS               ), // -
            util::ASCII_CODE_ASTERISK    => ScanResult::BcCode(util::ASCII_CODE_ASTERISK            ), // *
            util::ASCII_CODE_SLUSH       => ScanResult::BcCode(util::ASCII_CODE_SLUSH               ), // /
            util::ASCII_CODE_PERIOD      => ScanResult::BcCode(util::ASCII_CODE_PERIOD              ), // .
            util::ASCII_CODE_EQUAL       => ScanResult::BcCode(util::ASCII_CODE_EQUAL               ), // =
            util::ASCII_CODE_SEMICOLON   => ScanResult::BcCode(util::ASCII_CODE_SEMICOLON           ), // ;
            util::ASCII_CODE_NEWLINE     => ScanResult::EndCode,                                       // \n
            util::ASCII_CODE_ESCAPE      => ScanResult::EndCode,                                       // escape key
            util::ASCII_CODE_DELETE      => ScanResult::DeleteCode,                                    // delete key
            util::ASCII_CODE_SPACE       => ScanResult::BcCode(util::ASCII_CODE_SPACE               ), // white space key
            _                            => ScanResult::UnknownCode,
        }
    }

    fn delete_column(&mut self) {
        if &self.currnet_cur_pos > &0 {
            self.currnet_cur_pos -= 1;
            &self.input_vec.remove(self.currnet_cur_pos);
        }
    }

    fn cursor_move_left(&mut self) {
        if &self.currnet_cur_pos > &0 {
            self.currnet_cur_pos -= 1;
        }
    }

    fn cursor_move_right(&mut self) {
        if &self.currnet_cur_pos < &self.before_printed_statement_len {
            self.currnet_cur_pos += 1;
        }
    }

    fn insert_column(&mut self, code: &u8) {
        self.currnet_cur_pos += 1;
        &self.input_vec.insert(self.currnet_cur_pos-1, code.to_owned());
    }

    fn statement_from_utf8(&self) -> String {
        std::str::from_utf8(&self.input_vec).unwrap_or(&"error occured").to_string()
    }

    fn print_ok(&mut self, p1: &str, p2: &str, p3: &str, p4: &str) {
        self.before_printed_result_len = p4.to_string().len();
        self.before_printed_statement_len = p2.to_string().len();
        self.before_printed_len = p1.to_string().len() + self.before_printed_statement_len + p3.to_string().len() + self.before_printed_result_len;
        let pos_differnce = self.before_printed_statement_len - self.currnet_cur_pos;
        print!("{}{}{}{}{}{}", util::COLOR_CYAN_HEADER, p1, p2, p3, &p4, util::COLOR_PLAIN_HEADER);
        print!("{}{}{}", util::CURSOR_MOVE_ES_HEAD, (p3.to_string().len() + self.before_printed_result_len + pos_differnce), util::CURSOR_MOVE_ES_BACK);
    }

    fn print_ng(&mut self, p1: &str, p2: &str, p3: &str) {
        self.before_printed_statement_len = p2.to_string().len();
        self.before_printed_len = p1.to_string().len() +  self.before_printed_statement_len + p3.to_string().len() + self.before_printed_result_len;
        let pos_differnce =  self.before_printed_statement_len - &self.currnet_cur_pos;
        print!("{}{}{}{}{}", util::COLOR_MAGENDA_HEADER, p1, p2, p3, util::COLOR_PLAIN_HEADER);
        print!("{}{}{}", util::CURSOR_MOVE_ES_HEAD, (p3.to_string().len() + pos_differnce), util::CURSOR_MOVE_ES_BACK);
    }

    pub fn run(&mut self) {
        let ptr = &mut [0; 1];

        print!("{}", util::DNTK_PROMPT);
        loop {
            let r = unsafe { libc::read(0, ptr.as_ptr() as *mut libc::c_void, 1) };
            if r > 0 {
                let input_char = ptr[0] as u8;
                match &self.char_scan(input_char) {
                    ScanResult::UnknownCode => {
                        // TODO INFORM WARN
                    },
                    ScanResult::EndCode => {
                        print!("\n");
                        break
                    },
                    ScanResult::DeleteCode => {
                        &self.delete_column();
                    },
                    ScanResult::CurLeftCode => {
                        &self.cursor_move_left();
                    },
                    ScanResult::CurRightCode => {
                        &self.cursor_move_right();
                    },
                    ScanResult::BcCode(code) => {
                        &self.insert_column(code);
                    },
                }
                &self.fill_whitespace();
                let p1 = format!("{}", util::DNTK_PROMPT);
                let p2 = &self.statement_from_utf8();
                let p3 = " = ";
                match bc!(format!("{}", p2)) {
                    Ok(p4) => {
                        &self.print_ok(&p1, p2, p3, &p4);
                    },
                    _ => {
                        &self.print_ng(&p1, p2, p3);
                    },
                }
            }
            std::io::stdout().flush().unwrap();
        }
    }
}