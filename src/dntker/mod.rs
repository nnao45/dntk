mod util;

use std::io::Write;

use bc::{bc, BCError};

#[derive(Debug)]
pub struct Dntker {
    pub input_vec: Vec<u8>,
    pub before_printed_len: usize,
    pub before_printed_result_len: usize,
    pub before_printed_statement_len: usize,
    pub currnet_cur_pos: usize,
}

#[derive(Debug, PartialEq)]
enum FilterResult {
    BcCode(u8),
    EndCode,
    DeleteCode,
    CurLeftCode,
    CurRightCode,
    UnknownCode(u8),
}

#[derive(Debug)]
enum DntkResult {
    Output(String),
    Fin,
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

    fn filter_char(&self, ascii_char: u8) -> FilterResult {
        match ascii_char {
            util::ASCII_CODE_ZERO        => FilterResult::BcCode(util::ASCII_CODE_ZERO      ), // 0
            util::ASCII_CODE_ONE         => FilterResult::BcCode(util::ASCII_CODE_ONE       ), // 1
            util::ASCII_CODE_TWO         => FilterResult::BcCode(util::ASCII_CODE_TWO       ), // 2
            util::ASCII_CODE_THREE       => FilterResult::BcCode(util::ASCII_CODE_THREE     ), // 3
            util::ASCII_CODE_FOUR        => FilterResult::BcCode(util::ASCII_CODE_FOUR      ), // 4
            util::ASCII_CODE_FIVE        => FilterResult::BcCode(util::ASCII_CODE_FIVE      ), // 5
            util::ASCII_CODE_SIX         => FilterResult::BcCode(util::ASCII_CODE_SIX       ), // 6
            util::ASCII_CODE_SEVEN       => FilterResult::BcCode(util::ASCII_CODE_SEVEN     ), // 7
            util::ASCII_CODE_EIGHT       => FilterResult::BcCode(util::ASCII_CODE_EIGHT     ), // 8
            util::ASCII_CODE_NINE        => FilterResult::BcCode(util::ASCII_CODE_NINE      ), // 9
            util::ASCII_CODE_S           => FilterResult::BcCode(util::ASCII_CODE_S         ), // s
            util::ASCII_CODE_C           => FilterResult::BcCode(util::ASCII_CODE_C         ), // c
            util::ASCII_CODE_A           => FilterResult::BcCode(util::ASCII_CODE_A         ), // a
            util::ASCII_CODE_L           => FilterResult::BcCode(util::ASCII_CODE_L         ), // l
            util::ASCII_CODE_E           => FilterResult::BcCode(util::ASCII_CODE_E         ), // e
            util::ASCII_CODE_J           => FilterResult::BcCode(util::ASCII_CODE_J         ), // j
            util::ASCII_CODE_ROUNDLEFT   => FilterResult::BcCode(util::ASCII_CODE_ROUNDLEFT ), // (
            util::ASCII_CODE_ROUNDRIGHT  => FilterResult::BcCode(util::ASCII_CODE_ROUNDRIGHT), // )
            util::ASCII_CODE_SQUARELEFT  => FilterResult::CurLeftCode,                         // [
            util::ASCII_CODE_SQUARERIGHT => FilterResult::CurRightCode,                        // ]
            util::ASCII_CODE_PLUS        => FilterResult::BcCode(util::ASCII_CODE_PLUS      ), // +
            util::ASCII_CODE_MINUS       => FilterResult::BcCode(util::ASCII_CODE_MINUS     ), // -
            util::ASCII_CODE_ASTERISK    => FilterResult::BcCode(util::ASCII_CODE_ASTERISK  ), // *
            util::ASCII_CODE_SLUSH       => FilterResult::BcCode(util::ASCII_CODE_SLUSH     ), // /
            util::ASCII_CODE_PERIOD      => FilterResult::BcCode(util::ASCII_CODE_PERIOD    ), // .
            util::ASCII_CODE_EQUAL       => FilterResult::BcCode(util::ASCII_CODE_EQUAL     ), // =
            util::ASCII_CODE_SEMICOLON   => FilterResult::BcCode(util::ASCII_CODE_SEMICOLON ), // ;
            util::ASCII_CODE_NEWLINE     => FilterResult::EndCode,                             // \n
            util::ASCII_CODE_ESCAPE      => FilterResult::EndCode,                             // escape key
            util::ASCII_CODE_DELETE      => FilterResult::DeleteCode,                          // delete key
            util::ASCII_CODE_SPACE       => FilterResult::BcCode(util::ASCII_CODE_SPACE     ), // white space key
            unknown_code                 => FilterResult::UnknownCode(unknown_code),
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

    fn output_fill_whitespace(&self, len: usize) -> String {
        format!("\r{}", (0..len).map(|_| " ").collect::<String>())
    }

    fn output_ok(&mut self, p1: &str, p2: &str, p3: &str, p4: &str) -> String {
        self.before_printed_result_len = p4.to_string().len();
        self.before_printed_statement_len = p2.to_string().len();
        self.before_printed_len = p1.to_string().len() + self.before_printed_statement_len + p3.to_string().len() + self.before_printed_result_len;
        let pos_differnce = self.before_printed_statement_len - self.currnet_cur_pos;
        format!("{}{}{}{}{}{}{}{}{}", util::COLOR_CYAN_HEADER, p1, p2, p3, p4, util::COLOR_PLAIN_HEADER, util::CURSOR_MOVE_ES_HEAD, (p3.to_string().len() + self.before_printed_result_len + pos_differnce), util::CURSOR_MOVE_ES_BACK)
    }

    fn output_ng(&mut self, p1: &str, p2: &str, p3: &str) -> String {
        self.before_printed_statement_len = p2.to_string().len();
        self.before_printed_len = p1.to_string().len() +  self.before_printed_statement_len + p3.to_string().len() + self.before_printed_result_len;
        let pos_differnce =  self.before_printed_statement_len - &self.currnet_cur_pos;
        format!("{}{}{}{}{}{}{}{}", util::COLOR_MAGENDA_HEADER, p1, p2, p3, util::COLOR_PLAIN_HEADER, util::CURSOR_MOVE_ES_HEAD, (p3.to_string().len() + pos_differnce), util::CURSOR_MOVE_ES_BACK)
    }

    fn info_wn(&mut self, unknown_code: &u8) {
        print!("{}", self.output_fill_whitespace(self.before_printed_len));
        let warn_str =format!("this char is no supported: {}", unknown_code.to_owned() as char);
        print!("{}{}{}{}", "\r", util::COLOR_YELLOW_HEADER, warn_str, util::COLOR_PLAIN_HEADER);
        std::io::stdout().flush().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1000));
        print!("{}", &self.output_fill_whitespace(warn_str.len()));
    }

    fn dntk_exec(&mut self, ptr: [libc::c_char; 1]) -> DntkResult {
        let input_char = ptr[0] as u8;
        match &self.filter_char(input_char) {
            FilterResult::UnknownCode(unknown_code) => {
                &self.info_wn(unknown_code);
            },
            FilterResult::EndCode => {
                return DntkResult::Fin
            },
            FilterResult::DeleteCode => {
                &self.delete_column();
            },
            FilterResult::CurLeftCode => {
                &self.cursor_move_left();
            },
            FilterResult::CurRightCode => {
                &self.cursor_move_right();
            },
            FilterResult::BcCode(code) => {
                &self.insert_column(code);
            },
        }
        print!("{}", self.output_fill_whitespace(self.before_printed_len));
        let p1 = format!("{}", util::DNTK_PROMPT);
        let p2 = &self.statement_from_utf8();
        let p3 = " = ";
        match bc!(format!("{}", p2)) {
            Ok(p4) => {
                DntkResult::Output(self.output_ok(&p1, p2, p3, &p4))
            },
            Err(e) => {
                match e {
                    BCError::PopenError(e) => panic!("{:?}", e),
                    _ => {
                        DntkResult::Output(self.output_ng(&p1, p2, p3))
                    },
                }
            },
        }
    }

    pub fn run(&mut self) {
        let ptr: [libc::c_char; 1] = [0; 1];

        print!("{}", util::DNTK_PROMPT);
        loop {
            let r = unsafe { libc::read(0, ptr.as_ptr() as *mut libc::c_void, 1) };
            if r > 0 {
                match self.dntk_exec(ptr) {
                    DntkResult::Fin => {
                        print!("\n");
                        break
                    },
                    DntkResult::Output(o) => {
                        print!("{}", o);
                    },
                }
            }
            std::io::stdout().flush().unwrap();
        }
    }
}

#[cfg(test)]
mod dntker_tests {
    use super::{Dntker, util, FilterResult};
    #[test]
    fn test_filter_char() {
        let d = Dntker::new();
        assert_eq!(d.filter_char(util::ASCII_CODE_ZERO       ), FilterResult::BcCode(util::ASCII_CODE_ZERO      ));
        assert_eq!(d.filter_char(util::ASCII_CODE_ONE        ), FilterResult::BcCode(util::ASCII_CODE_ONE       ));
        assert_eq!(d.filter_char(util::ASCII_CODE_TWO        ), FilterResult::BcCode(util::ASCII_CODE_TWO       ));
        assert_eq!(d.filter_char(util::ASCII_CODE_THREE      ), FilterResult::BcCode(util::ASCII_CODE_THREE     ));
        assert_eq!(d.filter_char(util::ASCII_CODE_FOUR       ), FilterResult::BcCode(util::ASCII_CODE_FOUR      ));
        assert_eq!(d.filter_char(util::ASCII_CODE_FIVE       ), FilterResult::BcCode(util::ASCII_CODE_FIVE      ));
        assert_eq!(d.filter_char(util::ASCII_CODE_SIX        ), FilterResult::BcCode(util::ASCII_CODE_SIX       ));
        assert_eq!(d.filter_char(util::ASCII_CODE_SEVEN      ), FilterResult::BcCode(util::ASCII_CODE_SEVEN     ));
        assert_eq!(d.filter_char(util::ASCII_CODE_EIGHT      ), FilterResult::BcCode(util::ASCII_CODE_EIGHT     ));
        assert_eq!(d.filter_char(util::ASCII_CODE_NINE       ), FilterResult::BcCode(util::ASCII_CODE_NINE      ));
        assert_eq!(d.filter_char(util::ASCII_CODE_S          ), FilterResult::BcCode(util::ASCII_CODE_S         ));
        assert_eq!(d.filter_char(util::ASCII_CODE_C          ), FilterResult::BcCode(util::ASCII_CODE_C         ));
        assert_eq!(d.filter_char(util::ASCII_CODE_A          ), FilterResult::BcCode(util::ASCII_CODE_A         ));
        assert_eq!(d.filter_char(util::ASCII_CODE_L          ), FilterResult::BcCode(util::ASCII_CODE_L         ));
        assert_eq!(d.filter_char(util::ASCII_CODE_E          ), FilterResult::BcCode(util::ASCII_CODE_E         ));
        assert_eq!(d.filter_char(util::ASCII_CODE_J          ), FilterResult::BcCode(util::ASCII_CODE_J         ));
        assert_eq!(d.filter_char(util::ASCII_CODE_ROUNDLEFT  ), FilterResult::BcCode(util::ASCII_CODE_ROUNDLEFT ));
        assert_eq!(d.filter_char(util::ASCII_CODE_ROUNDRIGHT ), FilterResult::BcCode(util::ASCII_CODE_ROUNDRIGHT));
        assert_eq!(d.filter_char(util::ASCII_CODE_SQUARELEFT ), FilterResult::CurLeftCode                        );
        assert_eq!(d.filter_char(util::ASCII_CODE_SQUARERIGHT), FilterResult::CurRightCode                       );
        assert_eq!(d.filter_char(util::ASCII_CODE_PLUS       ), FilterResult::BcCode(util::ASCII_CODE_PLUS      ));
        assert_eq!(d.filter_char(util::ASCII_CODE_MINUS      ), FilterResult::BcCode(util::ASCII_CODE_MINUS     ));
        assert_eq!(d.filter_char(util::ASCII_CODE_ASTERISK   ), FilterResult::BcCode(util::ASCII_CODE_ASTERISK  ));
        assert_eq!(d.filter_char(util::ASCII_CODE_SLUSH      ), FilterResult::BcCode(util::ASCII_CODE_SLUSH     ));
        assert_eq!(d.filter_char(util::ASCII_CODE_PERIOD     ), FilterResult::BcCode(util::ASCII_CODE_PERIOD    ));
        assert_eq!(d.filter_char(util::ASCII_CODE_EQUAL      ), FilterResult::BcCode(util::ASCII_CODE_EQUAL     ));
        assert_eq!(d.filter_char(util::ASCII_CODE_SEMICOLON  ), FilterResult::BcCode(util::ASCII_CODE_SEMICOLON ));
        assert_eq!(d.filter_char(util::ASCII_CODE_NEWLINE    ), FilterResult::EndCode                            );
        assert_eq!(d.filter_char(util::ASCII_CODE_ESCAPE     ), FilterResult::EndCode                            );
        assert_eq!(d.filter_char(util::ASCII_CODE_DELETE     ), FilterResult::DeleteCode                         );
        assert_eq!(d.filter_char(util::ASCII_CODE_SPACE      ), FilterResult::BcCode(util::ASCII_CODE_SPACE     ));

        assert_eq!(d.filter_char(0x00                        ), FilterResult::UnknownCode(0x00                  ));
        assert_eq!(d.filter_char(0x21                        ), FilterResult::UnknownCode(0x21                  ));
        assert_eq!(d.filter_char(0x4f                        ), FilterResult::UnknownCode(0x4f                  ));
    }

    pub fn new_dummy_dntker(p1: usize, p2: usize, p3: usize, p4: usize) -> Dntker {
        Dntker {
            input_vec: vec![util::ASCII_CODE_ONE , util::ASCII_CODE_PLUS, util::ASCII_CODE_TWO],
            before_printed_len: p1,
            before_printed_result_len: p2,
            before_printed_statement_len: p3,
            currnet_cur_pos: p4,
        }
    }

    #[test]
    fn test_delete_column() {
        let d1 = &mut Dntker::new();
        d1.delete_column();

        assert_eq!(d1.input_vec, vec![]);
        assert_eq!(d1.currnet_cur_pos, 0);

        assert_eq!(d1.before_printed_len, 0);
        assert_eq!(d1.before_printed_statement_len, 0);

        let test_before_printed_len = 3;
        let test_before_printed_result_len = 1;
        let test_before_printed_statement_len = 3;
        let test_currnet_cur_pos = 3;
        let d2 = &mut new_dummy_dntker(
            test_before_printed_len,
            test_before_printed_result_len,
            test_before_printed_statement_len,
            test_currnet_cur_pos,
        );
        d2.delete_column();

        assert_eq!(d2.input_vec, vec![util::ASCII_CODE_ONE , util::ASCII_CODE_PLUS]);
        assert_eq!(d2.currnet_cur_pos, test_currnet_cur_pos-1);

        assert_eq!(d2.before_printed_len, test_before_printed_len);
        assert_eq!(d2.before_printed_statement_len, test_before_printed_statement_len);
    }

    #[test]
    fn test_cursor_move_left() {
        let d1 = &mut Dntker::new();
        d1.delete_column();

        assert_eq!(d1.input_vec, vec![]);
        assert_eq!(d1.currnet_cur_pos, 0);

        assert_eq!(d1.before_printed_len, 0);
        assert_eq!(d1.before_printed_statement_len, 0);
    }
}