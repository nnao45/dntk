mod util;

use std::io::Write;
use bc::{bc, BCError};
use atty::Stream;

#[derive(Debug, PartialEq)]
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
    RefreshCode,
    DeleteCode,
    CurLeftCode,
    CurRightCode,
    UnknownCode(u8),
}

#[derive(Debug, PartialEq)]
enum DntkResult {
    Output(String),
    Fin,
    Continue,
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
            util::ASCII_CODE_R           => FilterResult::RefreshCode,                         // r
            util::ASCII_CODE_ROUNDLEFT   => FilterResult::BcCode(util::ASCII_CODE_ROUNDLEFT ), // (
            util::ASCII_CODE_ROUNDRIGHT  => FilterResult::BcCode(util::ASCII_CODE_ROUNDRIGHT), // )
            util::ASCII_CODE_SQUARELEFT  => FilterResult::CurLeftCode,                         // [
            util::ASCII_CODE_SQUARERIGHT => FilterResult::CurRightCode,                        // ]
            util::ASCII_CODE_PLUS        => FilterResult::BcCode(util::ASCII_CODE_PLUS      ), // +
            util::ASCII_CODE_MINUS       => FilterResult::BcCode(util::ASCII_CODE_MINUS     ), // -
            util::ASCII_CODE_ASTERISK    => FilterResult::BcCode(util::ASCII_CODE_ASTERISK  ), // *
            util::ASCII_CODE_SLUSH       => FilterResult::BcCode(util::ASCII_CODE_SLUSH     ), // /
            util::ASCII_CODE_DOT         => FilterResult::BcCode(util::ASCII_CODE_DOT       ), // .
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
        std::str::from_utf8(&self.input_vec).unwrap().to_string()
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

    fn refresh(&mut self) {
        print!("{}", self.output_fill_whitespace(self.before_printed_len));
        *self = Dntker::new();
        print!("{}", util::DNTK_PROMPT);
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
            FilterResult::RefreshCode => {
                &self.refresh();
                return DntkResult::Continue
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
        if !atty::is(Stream::Stdin) {
            let mut s = String::new();
            std::io::stdin().read_line(&mut s).ok();
            println!("{}", bc!(s).unwrap());
            return
        };

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
                    DntkResult::Continue => {
                        continue
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
    use super::{Dntker, util, FilterResult, DntkResult};
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
        assert_eq!(d.filter_char(util::ASCII_CODE_DOT        ), FilterResult::BcCode(util::ASCII_CODE_DOT       ));
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

    #[test]
    fn test_delete_column() {
        let d1 = &mut Dntker::new();
        d1.delete_column();

        assert_eq!(d1.input_vec, vec![]);
        assert_eq!(d1.currnet_cur_pos, 0);

        assert_eq!(d1.before_printed_len, 0);
        assert_eq!(d1.before_printed_statement_len, 0);

        let test_input_vec = vec![util::ASCII_CODE_ONE , util::ASCII_CODE_PLUS, util::ASCII_CODE_TWO];
        let test_before_printed_len = 3;
        let test_before_printed_result_len = 1;
        let test_before_printed_statement_len = 3;
        let test_currnet_cur_pos = 2;
        let d2 = &mut  Dntker {
            input_vec: test_input_vec,
            before_printed_len: test_before_printed_len,
            before_printed_result_len: test_before_printed_result_len,
            before_printed_statement_len: test_before_printed_statement_len,
            currnet_cur_pos: test_currnet_cur_pos,
        };
        d2.delete_column();

        assert_eq!(d2.input_vec, vec![util::ASCII_CODE_ONE , util::ASCII_CODE_TWO]);
        assert_eq!(d2.currnet_cur_pos, test_currnet_cur_pos-1);

        assert_eq!(d2.before_printed_len, test_before_printed_len);
        assert_eq!(d2.before_printed_statement_len, test_before_printed_statement_len);
    }

    #[test]
    fn test_cursor_move_left() {
        let d1 = &mut Dntker::new();
        d1.cursor_move_left();

        assert_eq!(d1.input_vec, vec![]);
        assert_eq!(d1.currnet_cur_pos, 0);

        assert_eq!(d1.before_printed_len, 0);
        assert_eq!(d1.before_printed_result_len, 0);
        assert_eq!(d1.before_printed_statement_len, 0);

        let test_input_vec = vec![util::ASCII_CODE_ONE , util::ASCII_CODE_PLUS, util::ASCII_CODE_TWO];
        let test_before_printed_len = 3;
        let test_before_printed_result_len = 1;
        let test_before_printed_statement_len = 3;
        let test_currnet_cur_pos = 2;
        let d2 = &mut  Dntker {
            input_vec: test_input_vec,
            before_printed_len: test_before_printed_len,
            before_printed_result_len: test_before_printed_result_len,
            before_printed_statement_len: test_before_printed_statement_len,
            currnet_cur_pos: test_currnet_cur_pos,
        };
        d2.cursor_move_left();

        assert_eq!(d2.input_vec, vec![util::ASCII_CODE_ONE , util::ASCII_CODE_PLUS, util::ASCII_CODE_TWO]);
        assert_eq!(d2.currnet_cur_pos, 1);

        assert_eq!(d2.before_printed_len, 3);
        assert_eq!(d2.before_printed_result_len, 1);
        assert_eq!(d2.before_printed_statement_len, 3);
    }

    #[test]
    fn test_cursor_move_right() {
        let d1 = &mut Dntker::new();
        d1.cursor_move_right();

        assert_eq!(d1.input_vec, vec![]);
        assert_eq!(d1.currnet_cur_pos, 0);

        assert_eq!(d1.before_printed_len, 0);
        assert_eq!(d1.before_printed_result_len, 0);
        assert_eq!(d1.before_printed_statement_len, 0);

        let test_input_vec = vec![util::ASCII_CODE_ONE , util::ASCII_CODE_PLUS, util::ASCII_CODE_TWO];
        let test_before_printed_len = 3;
        let test_before_printed_result_len = 1;
        let test_before_printed_statement_len = 3;
        let test_currnet_cur_pos = 2;
        let d2 = &mut  Dntker {
            input_vec: test_input_vec,
            before_printed_len: test_before_printed_len,
            before_printed_result_len: test_before_printed_result_len,
            before_printed_statement_len: test_before_printed_statement_len,
            currnet_cur_pos: test_currnet_cur_pos,
        };
        d2.cursor_move_right();

        assert_eq!(d2.input_vec, vec![util::ASCII_CODE_ONE , util::ASCII_CODE_PLUS, util::ASCII_CODE_TWO]);
        assert_eq!(d2.currnet_cur_pos, 3);

        assert_eq!(d2.before_printed_len, 3);
        assert_eq!(d2.before_printed_result_len, 1);
        assert_eq!(d2.before_printed_statement_len, 3);
    }

    #[test]
    fn test_insert_column(){
        let d1 = &mut Dntker::new();
        let test_item = util::ASCII_CODE_THREE;
        d1.insert_column(&test_item);

        assert_eq!(d1.input_vec, vec![util::ASCII_CODE_THREE]);
        assert_eq!(d1.currnet_cur_pos, 1);

        assert_eq!(d1.before_printed_len, 0);
        assert_eq!(d1.before_printed_result_len, 0);
        assert_eq!(d1.before_printed_statement_len, 0);

        let test_input_vec = vec![util::ASCII_CODE_ONE , util::ASCII_CODE_PLUS, util::ASCII_CODE_TWO];
        let test_before_printed_len = 3;
        let test_before_printed_result_len = 1;
        let test_before_printed_statement_len = 3;
        let test_currnet_cur_pos = 2;
        let d2 = &mut  Dntker {
            input_vec: test_input_vec,
            before_printed_len: test_before_printed_len,
            before_printed_result_len: test_before_printed_result_len,
            before_printed_statement_len: test_before_printed_statement_len,
            currnet_cur_pos: test_currnet_cur_pos,
        };
        d2.insert_column(&test_item);

        assert_eq!(d2.input_vec, vec![util::ASCII_CODE_ONE , util::ASCII_CODE_PLUS, util::ASCII_CODE_THREE, util::ASCII_CODE_TWO]);
        assert_eq!(d2.currnet_cur_pos, 3);

        assert_eq!(d2.before_printed_len, 3);
        assert_eq!(d2.before_printed_result_len, 1);
        assert_eq!(d2.before_printed_statement_len, 3);
    }

    #[test]
    fn test_statement_from_utf8() {
        let test_input_vec1 = vec![util::ASCII_CODE_ONE , util::ASCII_CODE_PLUS, util::ASCII_CODE_TWO];
        let test_input_vec2 = vec![util::ASCII_CODE_S, util::ASCII_CODE_ROUNDLEFT, util::ASCII_CODE_EIGHT, util::ASCII_CODE_ROUNDRIGHT];
        let test_before_printed_len = 3;
        let test_before_printed_result_len = 1;
        let test_before_printed_statement_len = 3;
        let test_currnet_cur_pos = 2;
        let d1 = &mut Dntker {
            input_vec: test_input_vec1,
            before_printed_len: test_before_printed_len,
            before_printed_result_len: test_before_printed_result_len,
            before_printed_statement_len: test_before_printed_statement_len,
            currnet_cur_pos: test_currnet_cur_pos,
        };
        let d2 = &mut Dntker {
            input_vec: test_input_vec2,
            before_printed_len: test_before_printed_len,
            before_printed_result_len: test_before_printed_result_len,
            before_printed_statement_len: test_before_printed_statement_len,
            currnet_cur_pos: test_currnet_cur_pos,
        };
        assert_eq!("1+2".to_string(), d1.statement_from_utf8());
        assert_eq!("s(8)".to_string(), d2.statement_from_utf8());
    }

    #[test]
    fn test_output_fill_whitespace() {
        let d = Dntker::new();
        assert_eq!("\r".to_string(), d.output_fill_whitespace(0));
        assert_eq!("\r ".to_string(), d.output_fill_whitespace(1));
        assert_eq!("\r    ".to_string(), d.output_fill_whitespace(4));
    }

    #[test]
    fn test_output_ok() {
        let d = &mut Dntker::new();
        assert_eq!("\u{1b}[36m\r(dntk): 1+2 = 3\u{1b}[0m\u{1b}[7D".to_string(), d.output_ok(util::DNTK_PROMPT, "1+2", " = ", "3"));
        assert_eq!("\u{1b}[36m\r(dntk): a(123) = 1.56266642461495270762\u{1b}[0m\u{1b}[31D".to_string(), d.output_ok(util::DNTK_PROMPT, "a(123)", " = ", "1.56266642461495270762"));
    }

    #[test]
    fn test_output_ng() {
        let d = &mut Dntker::new();
        assert_eq!("\u{1b}[35m\r(dntk): 1+2* = \u{1b}[0m\u{1b}[7D".to_string(), d.output_ng(util::DNTK_PROMPT, "1+2*", " = "));
        assert_eq!("\u{1b}[35m\r(dntk): a(123)*s( = \u{1b}[0m\u{1b}[12D".to_string(), d.output_ng(util::DNTK_PROMPT, "a(123)*s(", " = "));
    }

    #[test]
    fn test_refresh() {
        let test_input_vec = vec![util::ASCII_CODE_ONE , util::ASCII_CODE_PLUS, util::ASCII_CODE_TWO];
        let test_before_printed_len = 3;
        let test_before_printed_result_len = 1;
        let test_before_printed_statement_len = 3;
        let test_currnet_cur_pos = 2;
        let d = &mut Dntker {
            input_vec: test_input_vec,
            before_printed_len: test_before_printed_len,
            before_printed_result_len: test_before_printed_result_len,
            before_printed_statement_len: test_before_printed_statement_len,
            currnet_cur_pos: test_currnet_cur_pos,
        };
        d.refresh();
        assert_eq!(d, &mut Dntker::new());
    }

    #[test]
    fn test_dntk_exec() {
        let d1 = &mut Dntker::new();
        let ptr_escape: [libc::c_char; 1] = [util::ASCII_CODE_ESCAPE as i8; 1];
        assert_eq!(DntkResult::Fin, d1.dntk_exec(ptr_escape));
        let ptr1: [libc::c_char; 1] = [util::ASCII_CODE_ONE as i8; 1];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1 = 1\u{1b}[0m\u{1b}[4D".to_string()), d1.dntk_exec(ptr1));
        let ptr2: [libc::c_char; 1] = [util::ASCII_CODE_PLUS as i8; 1];
        assert_eq!(DntkResult::Output("\u{1b}[35m\r(dntk): 1+ = \u{1b}[0m\u{1b}[3D".to_string()), d1.dntk_exec(ptr2));
        let ptr3: [libc::c_char; 1] = [util::ASCII_CODE_ZERO as i8; 1];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1+0 = 1\u{1b}[0m\u{1b}[4D".to_string()), d1.dntk_exec(ptr3));
        let ptr4: [libc::c_char; 1] = [util::ASCII_CODE_DOT as i8; 1];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1+0. = 1\u{1b}[0m\u{1b}[4D".to_string()), d1.dntk_exec(ptr4));
        let ptr_unknown_ascii: [libc::c_char; 1] = [0x4f as i8; 1];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1+0. = 1\u{1b}[0m\u{1b}[4D".to_string()), d1.dntk_exec(ptr_unknown_ascii));
        let ptr5: [libc::c_char; 1] = [util::ASCII_CODE_SEVEN as i8; 1];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1+0.7 = 1.7\u{1b}[0m\u{1b}[6D".to_string()), d1.dntk_exec(ptr5));
        let ptr_enter: [libc::c_char; 1] = [util::ASCII_CODE_NEWLINE as i8; 1];
        assert_eq!(DntkResult::Fin, d1.dntk_exec(ptr_enter));
    }
}