mod util;
mod bc;

use super::meta;
use std::io::Write;
use atty::Stream;

#[cfg(target_os = "windows")]
use winconsole::console as wconsole;

#[cfg(target_os = "windows")]
use ansi_term;

#[derive(Debug, PartialEq)]
pub struct Dntker {
    pub executer: bc::BcExecuter,
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
    EscCode,
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

struct DntkString {
    data: String,
    dtype: DntkStringType,
    cur_pos_from_right: usize,
}

enum DntkStringType {
    Ok,
    Ng,
    Warn,
    Refresh,
}

impl DntkString {
    pub fn ancize(mut self) -> Self {
        self = self.colorize();
        #[cfg(not(target_os = "windows"))]
        {
            self = self.cursorize();
        }
        self
    }

    pub fn colorize(mut self) -> Self {
        #[cfg(not(target_os = "windows"))]
        match &self.dtype {
            DntkStringType::Ok => {
                self.data = format!("{}{}{}", util::COLOR_CYAN_HEADER, &self.data, util::COLOR_PLAIN_HEADER);
            },
            DntkStringType::Ng => {
                self.data = format!("{}{}{}", util::COLOR_MAGENDA_HEADER, &self.data, util::COLOR_PLAIN_HEADER);
            },
            DntkStringType::Warn => {
                self.data = format!("{}{}{}", util::COLOR_YELLOW_HEADER, &self.data, util::COLOR_PLAIN_HEADER);
            },
            DntkStringType::Refresh => {
                self.data = format!("{}{}{}", util::COLOR_GREEN_HEADER, &self.data, util::COLOR_PLAIN_HEADER);
            },
        }
        #[cfg(target_os = "windows")]
        match &self.dtype {
            DntkStringType::Ok => {
                self.data = ansi_term::Colour::Cyan.paint(&self.data).to_string();
            },
            DntkStringType::Ng => {
                self.data = ansi_term::Colour::Purple.paint(&self.data).to_string();
            },
            DntkStringType::Warn => {
                self.data = ansi_term::Colour::Yellow.paint(&self.data).to_string();
            },
        }
        self
    }

    pub fn cursorize(mut self) -> Self {
        self.data = format!("{}{}{}{}", &self.data, util::CURSOR_MOVE_ES_HEAD, &self.cur_pos_from_right, util::CURSOR_MOVE_ES_BACK);
        self
    }

    pub fn to_string(self) -> String {
        format!("{}", &self.data)
    }
}

impl Dntker {
    pub fn new() -> Self {
        Dntker {
            executer: bc::BcExecuter::new(),
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
            util::ASCII_CODE_R           => FilterResult::BcCode(util::ASCII_CODE_R         ), // r
            util::ASCII_CODE_Q           => FilterResult::BcCode(util::ASCII_CODE_Q         ), // q
            util::ASCII_CODE_T           => FilterResult::BcCode(util::ASCII_CODE_T         ), // t
            util::ASCII_CODE_ROUNDLEFT   => FilterResult::BcCode(util::ASCII_CODE_ROUNDLEFT ), // (
            util::ASCII_CODE_ROUNDRIGHT  => FilterResult::BcCode(util::ASCII_CODE_ROUNDRIGHT), // )
            util::ASCII_CODE_SQUARELEFT  => FilterResult::CurLeftCode,                         // [
            util::ASCII_CODE_SQUARERIGHT => FilterResult::CurRightCode,                        // ]
            util::ASCII_CODE_LARGER      => FilterResult::BcCode(util::ASCII_CODE_LARGER    ), // <
            util::ASCII_CODE_SMALLER     => FilterResult::BcCode(util::ASCII_CODE_SMALLER   ), // >
            util::ASCII_CODE_PLUS        => FilterResult::BcCode(util::ASCII_CODE_PLUS      ), // +
            util::ASCII_CODE_MINUS       => FilterResult::BcCode(util::ASCII_CODE_MINUS     ), // -
            util::ASCII_CODE_ASTERISK    => FilterResult::BcCode(util::ASCII_CODE_ASTERISK  ), // *
            util::ASCII_CODE_SLUSH       => FilterResult::BcCode(util::ASCII_CODE_SLUSH     ), // /
            util::ASCII_CODE_HAT         => FilterResult::BcCode(util::ASCII_CODE_HAT       ), // ^
            util::ASCII_CODE_PERCENT     => FilterResult::BcCode(util::ASCII_CODE_PERCENT   ), // %
            util::ASCII_CODE_DOT         => FilterResult::BcCode(util::ASCII_CODE_DOT       ), // .
            util::ASCII_CODE_BIKKURI     => FilterResult::BcCode(util::ASCII_CODE_BIKKURI   ), // !
            util::ASCII_CODE_EQUAL       => FilterResult::BcCode(util::ASCII_CODE_EQUAL     ), // =
            util::ASCII_CODE_PIPE        => FilterResult::BcCode(util::ASCII_CODE_PIPE      ), // |
            util::ASCII_CODE_AND         => FilterResult::BcCode(util::ASCII_CODE_AND       ), // &
            util::ASCII_CODE_SEMICOLON   => FilterResult::BcCode(util::ASCII_CODE_SEMICOLON ), // ;
            util::ASCII_CODE_AT          => FilterResult::RefreshCode,                         // @
            util::ASCII_CODE_NEWLINE     => FilterResult::EndCode,                             // \n
            util::ASCII_CODE_ESCAPE      => FilterResult::EscCode,                             // escape key
            util::ASCII_CODE_BACKSPACE   => FilterResult::DeleteCode,                          // backspace key
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

    fn statement_from_utf8(&mut self) -> String {
        std::str::from_utf8(&self.input_vec).unwrap().to_string()
    }

    fn output_fill_whitespace(&self, len: usize) -> String {
        format!("\r{}", (0..len).map(|_| " ").collect::<String>())
    }

    fn output_ok(&mut self, p1: &str, p2: &str, p3: &str, p4: &str) -> DntkString {
        self.before_printed_result_len = p4.to_string().len();
        self.before_printed_statement_len = p2.to_string().len();
        self.before_printed_len = p1.to_string().len() + self.before_printed_statement_len + p3.to_string().len() + self.before_printed_result_len;
        let pos_differnce = self.before_printed_statement_len - self.currnet_cur_pos;
        DntkString {
            data: format!("{}{}{}{}", p1, p2, p3, p4),
            dtype: DntkStringType::Ok,
            cur_pos_from_right: (p3.to_string().len() + self.before_printed_result_len + &pos_differnce),
        }
    }

    fn output_ng(&mut self, p1: &str, p2: &str, p3: &str) -> DntkString {
        self.before_printed_statement_len = p2.to_string().len();
        self.before_printed_len = p1.to_string().len() +  self.before_printed_statement_len + p3.to_string().len() + self.before_printed_result_len;
        let pos_differnce =  self.before_printed_statement_len - &self.currnet_cur_pos;
        DntkString {
            data: format!("{}{}{}", p1, p2, p3),
            dtype: DntkStringType::Ng,
            cur_pos_from_right: (p3.to_string().len() + &pos_differnce),
        }
    }

    fn inform(&mut self, msg: &str, dtype: DntkStringType) {
        if ! meta::build_cli().get_matches().is_present("quiet") {
            print!("{}", self.output_fill_whitespace(self.before_printed_len));
            print!("{}", DntkString {
                data: format!("{}{}", "\r", msg),
                dtype: dtype,
                cur_pos_from_right: 0,
            }
                .colorize()
                .to_string()
            );
            std::io::stdout().flush().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(1000));
            print!("{}", &self.output_fill_whitespace(msg.len()));
        }
    }

    fn warning(&mut self, unknown_code: &u8) {
        self.inform(&format!("this char is no supported: {}", unknown_code.to_owned() as char), DntkStringType::Warn)
    }

    fn refresh(&mut self) {
        self.inform("refresh!!", DntkStringType::Refresh);
        print!("{}", self.output_fill_whitespace(self.before_printed_len));
        *self = Dntker::new();
        print!("{}", util::DNTK_PROMPT);
        std::io::stdout().flush().unwrap();
    }

    fn dntk_exec(&mut self, ptr: [libc::c_char; 3]) -> DntkResult {
        let input_char = ptr[0] as u8;
        match &self.filter_char(input_char) {
            FilterResult::UnknownCode(unknown_code) => {
                &self.warning(unknown_code);
            },
            FilterResult::EscCode => {
                if ptr.len() >= 3 {
                    match ptr[2] as u8 {
                        util::ASCII_CODE_RIGHT => {
                            &self.cursor_move_right();
                        },
                        util::ASCII_CODE_LEFT => {
                            &self.cursor_move_left();
                        },
                        _ => {
                            return DntkResult::Fin
                        }
                    }
                } else {
                    return DntkResult::Fin
                }
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
        match &self.executer.exec(p2) {
            Ok(p4) => {
                DntkResult::Output(self.output_ok(&p1, p2, p3, &p4)
                                     .ancize()
                                     .to_string())
            },
            Err(e) => {
                match e {
                    bc::BcError::PopenError(e) => panic!("call bc process open error: {:?}", e),
                    bc::BcError::Timeout => panic!("call bc process is timeout"),
                    _ => {
                        DntkResult::Output(self.output_ng(&p1, p2, p3)
                                     .ancize()
                                     .to_string())
                    },
                }
            },
        }
    }

    #[cfg(target_os = "windows")]
    pub fn watch(&self,  mut ptr: [libc::c_char; 3]) -> [libc::c_char; 3] {
        ptr[0] = wconsole::getch(true).unwrap() as u8 as i8;
        return ptr
    }

    #[cfg(target_os = "macos")]
    pub fn watch(&self,  ptr: [libc::c_char; 3]) -> [libc::c_char; 3] {
        loop{
            if unsafe { libc::read(0, ptr.as_ptr() as *mut libc::c_void, 3) } > 0 {
                return ptr
            };
        }
    }

    pub fn run(&mut self) {
        if !atty::is(Stream::Stdin) {
            let mut s = String::new();
            std::io::stdin().read_line(&mut s).ok();
            println!("{}", &self.executer.exec(&s).unwrap());
            return
        };

        let ptr: [libc::c_char; 3] = [0; 3];

        print!("{}", util::DNTK_PROMPT);
        std::io::stdout().flush().unwrap();
        loop {
            match self.dntk_exec(self.watch(ptr)) {
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
            std::io::stdout().flush().unwrap();
            #[cfg(target_os = "windows")]
            {
                let vec_cur = wconsole::get_cursor_position().unwrap();
                wconsole::set_cursor_position(util::DNTK_PROMPT.to_string().len() as u16 + self.currnet_cur_pos as u16 -1, vec_cur.y).unwrap();
            }
        }
    }
}

#[cfg(test)]
mod dntker_tests {
    use super::{Dntker, util, FilterResult, DntkResult, bc, DntkString, DntkStringType};
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
        assert_eq!(d.filter_char(util::ASCII_CODE_R          ), FilterResult::BcCode(util::ASCII_CODE_R         ));
        assert_eq!(d.filter_char(util::ASCII_CODE_Q          ), FilterResult::BcCode(util::ASCII_CODE_Q         ));
        assert_eq!(d.filter_char(util::ASCII_CODE_T          ), FilterResult::BcCode(util::ASCII_CODE_T         ));
        assert_eq!(d.filter_char(util::ASCII_CODE_ROUNDLEFT  ), FilterResult::BcCode(util::ASCII_CODE_ROUNDLEFT ));
        assert_eq!(d.filter_char(util::ASCII_CODE_ROUNDRIGHT ), FilterResult::BcCode(util::ASCII_CODE_ROUNDRIGHT));
        assert_eq!(d.filter_char(util::ASCII_CODE_LARGER     ), FilterResult::BcCode(util::ASCII_CODE_LARGER    ));
        assert_eq!(d.filter_char(util::ASCII_CODE_SMALLER    ), FilterResult::BcCode(util::ASCII_CODE_SMALLER   ));
        assert_eq!(d.filter_char(util::ASCII_CODE_SQUARELEFT ), FilterResult::CurLeftCode                        );
        assert_eq!(d.filter_char(util::ASCII_CODE_SQUARERIGHT), FilterResult::CurRightCode                       );
        assert_eq!(d.filter_char(util::ASCII_CODE_PLUS       ), FilterResult::BcCode(util::ASCII_CODE_PLUS      ));
        assert_eq!(d.filter_char(util::ASCII_CODE_MINUS      ), FilterResult::BcCode(util::ASCII_CODE_MINUS     ));
        assert_eq!(d.filter_char(util::ASCII_CODE_ASTERISK   ), FilterResult::BcCode(util::ASCII_CODE_ASTERISK  ));
        assert_eq!(d.filter_char(util::ASCII_CODE_SLUSH      ), FilterResult::BcCode(util::ASCII_CODE_SLUSH     ));
        assert_eq!(d.filter_char(util::ASCII_CODE_HAT        ), FilterResult::BcCode(util::ASCII_CODE_HAT       ));
        assert_eq!(d.filter_char(util::ASCII_CODE_PERCENT    ), FilterResult::BcCode(util::ASCII_CODE_PERCENT   ));
        assert_eq!(d.filter_char(util::ASCII_CODE_DOT        ), FilterResult::BcCode(util::ASCII_CODE_DOT       ));
        assert_eq!(d.filter_char(util::ASCII_CODE_BIKKURI    ), FilterResult::BcCode(util::ASCII_CODE_BIKKURI   ));
        assert_eq!(d.filter_char(util::ASCII_CODE_EQUAL      ), FilterResult::BcCode(util::ASCII_CODE_EQUAL     ));
        assert_eq!(d.filter_char(util::ASCII_CODE_PIPE       ), FilterResult::BcCode(util::ASCII_CODE_PIPE      ));
        assert_eq!(d.filter_char(util::ASCII_CODE_AND        ), FilterResult::BcCode(util::ASCII_CODE_AND       ));
        assert_eq!(d.filter_char(util::ASCII_CODE_SEMICOLON  ), FilterResult::BcCode(util::ASCII_CODE_SEMICOLON ));
        assert_eq!(d.filter_char(util::ASCII_CODE_AT         ), FilterResult::RefreshCode                        );
        assert_eq!(d.filter_char(util::ASCII_CODE_NEWLINE    ), FilterResult::EndCode                            );
        assert_eq!(d.filter_char(util::ASCII_CODE_ESCAPE     ), FilterResult::EscCode                            );
        assert_eq!(d.filter_char(util::ASCII_CODE_BACKSPACE  ), FilterResult::DeleteCode                         );
        assert_eq!(d.filter_char(util::ASCII_CODE_DELETE     ), FilterResult::DeleteCode                         );
        assert_eq!(d.filter_char(util::ASCII_CODE_SPACE      ), FilterResult::BcCode(util::ASCII_CODE_SPACE     ));

        assert_eq!(d.filter_char(0x00                        ), FilterResult::UnknownCode(0x00                  ));
        assert_eq!(d.filter_char(0x0e                        ), FilterResult::UnknownCode(0x0e                  ));
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
        let d2 = &mut Dntker {
            executer: bc::BcExecuter::new(),
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
        let d2 = &mut Dntker {
            executer: bc::BcExecuter::new(),
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
        let d2 = &mut Dntker {
            executer: bc::BcExecuter::new(),
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
        let d2 = &mut Dntker {
            executer: bc::BcExecuter::new(),
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
            executer: bc::BcExecuter::new(),
            input_vec: test_input_vec1,
            before_printed_len: test_before_printed_len,
            before_printed_result_len: test_before_printed_result_len,
            before_printed_statement_len: test_before_printed_statement_len,
            currnet_cur_pos: test_currnet_cur_pos,
        };
        let d2 = &mut Dntker {
            executer: bc::BcExecuter::new(),
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
        assert_eq!("\u{1b}[36m\r(dntk): 1+2 = 3\u{1b}[0m\u{1b}[7D".to_string(), d.output_ok(util::DNTK_PROMPT, "1+2", " = ", "3").ancize().to_string());
        assert_eq!("\u{1b}[36m\r(dntk): a(123) = 1.56266642461495270762\u{1b}[0m\u{1b}[31D".to_string(), d.output_ok(util::DNTK_PROMPT, "a(123)", " = ", "1.56266642461495270762").ancize().to_string());
    }

    #[test]
    fn test_output_ng() {
        let d = &mut Dntker::new();
        assert_eq!("\u{1b}[35m\r(dntk): 1+2* = \u{1b}[0m\u{1b}[7D".to_string(), d.output_ng(util::DNTK_PROMPT, "1+2*", " = ").ancize().to_string());
        assert_eq!("\u{1b}[35m\r(dntk): a(123)*s( = \u{1b}[0m\u{1b}[12D".to_string(), d.output_ng(util::DNTK_PROMPT, "a(123)*s(", " = ").ancize().to_string());
    }

    #[test]
    fn test_refresh() {
        let test_input_vec = vec![util::ASCII_CODE_ONE , util::ASCII_CODE_PLUS, util::ASCII_CODE_TWO];
        let test_before_printed_len = 3;
        let test_before_printed_result_len = 1;
        let test_before_printed_statement_len = 3;
        let test_currnet_cur_pos = 2;
        let d = &mut Dntker {
            executer: bc::BcExecuter::new(),
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
        let ptr_escape: [libc::c_char; 3] = [util::ASCII_CODE_ESCAPE as i8; 3];
        assert_eq!(DntkResult::Fin, d1.dntk_exec(ptr_escape));
        let ptr1: [libc::c_char; 3] = [util::ASCII_CODE_ONE as i8; 3];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1 = 1\u{1b}[0m\u{1b}[4D".to_string()), d1.dntk_exec(ptr1));
        let ptr_right: [libc::c_char; 3] = [util::ASCII_CODE_ESCAPE as i8, 0x91 as u8 as i8, util::ASCII_CODE_RIGHT as i8];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1 = 1\u{1b}[0m\u{1b}[4D".to_string()), d1.dntk_exec(ptr_right));
        let ptr2: [libc::c_char; 3] = [util::ASCII_CODE_PLUS as i8; 3];
        assert_eq!(DntkResult::Output("\u{1b}[35m\r(dntk): 1+ = \u{1b}[0m\u{1b}[3D".to_string()), d1.dntk_exec(ptr2));
        let ptr3: [libc::c_char; 3] = [util::ASCII_CODE_ZERO as i8; 3];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1+0 = 1\u{1b}[0m\u{1b}[4D".to_string()), d1.dntk_exec(ptr3));
        let ptr4: [libc::c_char; 3] = [util::ASCII_CODE_DOT as i8; 3];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1+0. = 1\u{1b}[0m\u{1b}[4D".to_string()), d1.dntk_exec(ptr4));
        let ptr_unknown_ascii: [libc::c_char; 3] = [0x4f as i8; 3];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1+0. = 1\u{1b}[0m\u{1b}[4D".to_string()), d1.dntk_exec(ptr_unknown_ascii));
        let ptr5: [libc::c_char; 3] = [util::ASCII_CODE_SEVEN as i8; 3];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1+0.7 = 1.7\u{1b}[0m\u{1b}[6D".to_string()), d1.dntk_exec(ptr5));
        let ptr_enter: [libc::c_char; 3] = [util::ASCII_CODE_NEWLINE as i8; 3];
        assert_eq!(DntkResult::Fin, d1.dntk_exec(ptr_enter));
    }

    #[test]
    fn test_colorize() {
        let s = "\r(dntk): 1+2 = 3";
        let ds1 = DntkString {
            data: s.to_string(),
            dtype: DntkStringType::Ok,
            cur_pos_from_right: 4,
        };
        let ds2 = DntkString {
            data: s.to_string(),
            dtype: DntkStringType::Ng,
            cur_pos_from_right: 4,
        };
        let ds3 = DntkString {
            data: s.to_string(),
            dtype: DntkStringType::Warn,
            cur_pos_from_right: 4,
        };
        assert_eq!("\u{1b}[36m\r(dntk): 1+2 = 3\u{1b}[0m".to_string(), ds1.colorize().data);
        assert_eq!("\u{1b}[35m\r(dntk): 1+2 = 3\u{1b}[0m".to_string(), ds2.colorize().data);
        assert_eq!("\u{1b}[33m\r(dntk): 1+2 = 3\u{1b}[0m".to_string(), ds3.colorize().data);
    }

    #[test]
    fn test_cursorize() {
        let s = "\u{1b}[36m\r(dntk): 1+2 = 3\u{1b}[0m";
        let ds = DntkString {
            data: s.to_string(),
            dtype: DntkStringType::Ok,
            cur_pos_from_right: 4,
        };
        assert_eq!("\u{1b}[36m\r(dntk): 1+2 = 3\u{1b}[0m\u{1b}[4D".to_string(), ds.cursorize().data);
    }
}
