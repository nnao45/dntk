#[derive(Debug, PartialEq)]
pub struct Dntker {
    pub executer: bc::BcExecuter,
    pub input_vec: Vec<u8>,
    pub before_printed_len: usize,
    pub before_printed_result_len: usize,
    pub before_printed_statement_len: usize,
    pub currnet_cur_pos: usize,
}

impl Default for Dntker {
    fn default() -> Self {
        let mut iv : Vec<u8> = Vec::new();
        let mut bpsl = 0;
        let mut ccp = 0;
        if util::DNTK_OPT.inject != "" {
            let inject_bytes = &mut util::DNTK_OPT.inject.as_bytes().to_owned();
            bpsl = inject_bytes.len();
            ccp = inject_bytes.len();
            iv.append(inject_bytes);
        }
        Dntker {
            input_vec: iv,
            before_printed_statement_len: bpsl,
            currnet_cur_pos: ccp,

            executer: Default::default(),
            before_printed_len: Default::default(),
            before_printed_result_len: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq)]
enum FilterResult {
    Calculatable(u8),
    End,
    Esc,
    Refresh,
    Delete,
    CurLeft,
    CurRight,
    Unknown(u8),
}

#[derive(Debug, PartialEq)]
enum DntkResult {
    Output(String),
    Fin,
    Continue,
}

#[allow(dead_code)]
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
        if ! util::DNTK_OPT.white {
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
                DntkStringType::Refresh => {
                    self.data = ansi_term::Colour::Green.paint(&self.data).to_string();
                },
            }
        }
        self
    }

    #[allow(dead_code)]
    pub fn cursorize(mut self) -> Self {
        self.data = format!("{}{}{}{}", &self.data, util::CURSOR_MOVE_ES_HEAD, &self.cur_pos_from_right, util::CURSOR_MOVE_ES_BACK);
        self
    }
}

impl ToString for DntkString {
    fn to_string(&self) -> String {
        self.data.to_string()
    }
}

impl Dntker {
    #[allow(unused_must_use)]
    fn write_stdout(&self, buf: &str) {
        let out = stdout();
        let mut out = BufWriter::new(out.lock());
        write!(out, "{}", buf);
    }

    fn write_stdout_ln(&self, buf: &str) {
        self.write_stdout(&format!("{}\n", buf))
    }

    fn filter_char(&self, ascii_char: u8) -> FilterResult {
        match ascii_char {
            util::ASCII_CODE_ZERO        => FilterResult::Calculatable(util::ASCII_CODE_ZERO      ), // 0
            util::ASCII_CODE_ONE         => FilterResult::Calculatable(util::ASCII_CODE_ONE       ), // 1
            util::ASCII_CODE_TWO         => FilterResult::Calculatable(util::ASCII_CODE_TWO       ), // 2
            util::ASCII_CODE_THREE       => FilterResult::Calculatable(util::ASCII_CODE_THREE     ), // 3
            util::ASCII_CODE_FOUR        => FilterResult::Calculatable(util::ASCII_CODE_FOUR      ), // 4
            util::ASCII_CODE_FIVE        => FilterResult::Calculatable(util::ASCII_CODE_FIVE      ), // 5
            util::ASCII_CODE_SIX         => FilterResult::Calculatable(util::ASCII_CODE_SIX       ), // 6
            util::ASCII_CODE_SEVEN       => FilterResult::Calculatable(util::ASCII_CODE_SEVEN     ), // 7
            util::ASCII_CODE_EIGHT       => FilterResult::Calculatable(util::ASCII_CODE_EIGHT     ), // 8
            util::ASCII_CODE_NINE        => FilterResult::Calculatable(util::ASCII_CODE_NINE      ), // 9
            util::ASCII_CODE_S           => FilterResult::Calculatable(util::ASCII_CODE_S         ), // s
            util::ASCII_CODE_C           => FilterResult::Calculatable(util::ASCII_CODE_C         ), // c
            util::ASCII_CODE_A           => FilterResult::Calculatable(util::ASCII_CODE_A         ), // a
            util::ASCII_CODE_L           => FilterResult::Calculatable(util::ASCII_CODE_L         ), // l
            util::ASCII_CODE_E           => FilterResult::Calculatable(util::ASCII_CODE_E         ), // e
            util::ASCII_CODE_J           => FilterResult::Calculatable(util::ASCII_CODE_J         ), // j
            util::ASCII_CODE_R           => FilterResult::Calculatable(util::ASCII_CODE_R         ), // r
            util::ASCII_CODE_Q           => FilterResult::Calculatable(util::ASCII_CODE_Q         ), // q
            util::ASCII_CODE_T           => FilterResult::Calculatable(util::ASCII_CODE_T         ), // t
            util::ASCII_CODE_ROUNDLEFT   => FilterResult::Calculatable(util::ASCII_CODE_ROUNDLEFT ), // (
            util::ASCII_CODE_ROUNDRIGHT  => FilterResult::Calculatable(util::ASCII_CODE_ROUNDRIGHT), // )
            util::ASCII_CODE_SQUARELEFT  => FilterResult::CurLeft,                         // [
            util::ASCII_CODE_SQUARERIGHT => FilterResult::CurRight,                        // ]
            util::ASCII_CODE_LARGER      => FilterResult::Calculatable(util::ASCII_CODE_LARGER    ), // <
            util::ASCII_CODE_SMALLER     => FilterResult::Calculatable(util::ASCII_CODE_SMALLER   ), // >
            util::ASCII_CODE_PLUS        => FilterResult::Calculatable(util::ASCII_CODE_PLUS      ), // +
            util::ASCII_CODE_MINUS       => FilterResult::Calculatable(util::ASCII_CODE_MINUS     ), // -
            util::ASCII_CODE_ASTERISK    => FilterResult::Calculatable(util::ASCII_CODE_ASTERISK  ), // *
            util::ASCII_CODE_SLUSH       => FilterResult::Calculatable(util::ASCII_CODE_SLUSH     ), // /
            util::ASCII_CODE_HAT         => FilterResult::Calculatable(util::ASCII_CODE_HAT       ), // ^
            util::ASCII_CODE_PERCENT     => FilterResult::Calculatable(util::ASCII_CODE_PERCENT   ), // %
            util::ASCII_CODE_DOT         => FilterResult::Calculatable(util::ASCII_CODE_DOT       ), // .
            util::ASCII_CODE_COMMA       => FilterResult::Calculatable(util::ASCII_CODE_COMMA     ), // ,
            util::ASCII_CODE_BIKKURI     => FilterResult::Calculatable(util::ASCII_CODE_BIKKURI   ), // !
            util::ASCII_CODE_EQUAL       => FilterResult::Calculatable(util::ASCII_CODE_EQUAL     ), // =
            util::ASCII_CODE_PIPE        => FilterResult::Calculatable(util::ASCII_CODE_PIPE      ), // |
            util::ASCII_CODE_AND         => FilterResult::Calculatable(util::ASCII_CODE_AND       ), // &
            util::ASCII_CODE_SEMICOLON   => FilterResult::Calculatable(util::ASCII_CODE_SEMICOLON ), // ;
            util::ASCII_CODE_AT          => FilterResult::Refresh,                         // @
            util::ASCII_CODE_WINENTER    => FilterResult::End,                             // windows \n
            util::ASCII_CODE_NEWLINE     => FilterResult::End,                             // \n
            util::ASCII_CODE_ESCAPE      => FilterResult::Esc,                             // escape key
            util::ASCII_CODE_BACKSPACE   => FilterResult::Delete,                          // backspace key
            util::ASCII_CODE_DELETE      => FilterResult::Delete,                          // delete key
            util::ASCII_CODE_SPACE       => FilterResult::Calculatable(util::ASCII_CODE_SPACE     ), // white space key
            unknown_code                 => FilterResult::Unknown(unknown_code),
        }
    }

    fn delete_column(&mut self) {
        if self.currnet_cur_pos > 0 {
            self.currnet_cur_pos -= 1;
            self.input_vec.remove(self.currnet_cur_pos);
        }
    }

    fn cursor_move_left(&mut self) {
        if self.currnet_cur_pos > 0 {
            self.currnet_cur_pos -= 1;
        }
    }

    fn cursor_move_right(&mut self) {
        if self.currnet_cur_pos < self.before_printed_statement_len {
            self.currnet_cur_pos += 1;
        }
    }

    fn insert_column(&mut self, code: u8) {
        self.currnet_cur_pos += 1;
        self.input_vec.insert(self.currnet_cur_pos-1, code);
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
            cur_pos_from_right: (p3.to_string().len() + self.before_printed_result_len + pos_differnce),
        }
    }

    fn output_ng(&mut self, p1: &str, p2: &str, p3: &str) -> DntkString {
        self.before_printed_statement_len = p2.to_string().len();
        self.before_printed_len = p1.to_string().len() +  self.before_printed_statement_len + p3.to_string().len() + self.before_printed_result_len;
        let pos_differnce =  self.before_printed_statement_len - self.currnet_cur_pos;
        DntkString {
            data: format!("{}{}{}", p1, p2, p3),
            dtype: DntkStringType::Ng,
            cur_pos_from_right: (p3.to_string().len() + pos_differnce),
        }
    }

    fn inform(&mut self, msg: &str, dtype: DntkStringType) {
        if ! util::DNTK_OPT.quiet {
            self.write_stdout(&self.output_fill_whitespace(self.before_printed_len));
            self.write_stdout(DntkString {
                data: format!("{}{}", "\r", msg),
                dtype: dtype,
                cur_pos_from_right: 0,
            }
                .colorize()
                .to_string()
                .as_ref()
            );
            std::io::stdout().flush().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(1000));
            self.write_stdout(&self.output_fill_whitespace(msg.len()));
        }
    }

    fn warning(&mut self, unknown_code: u8) {
        self.inform(&format!("this char is no supported: {}", unknown_code as char), DntkStringType::Warn)
    }

    fn refresh(&mut self) {
        self.inform("refresh!!", DntkStringType::Refresh);
        self.write_stdout( &self.output_fill_whitespace(self.before_printed_len));
        let dnew: Dntker = Default::default();
        *self = dnew;
        self.write_stdout(util::DNTK_PROMPT);
        std::io::stdout().flush().unwrap();
    }

    fn calculate(&mut self, p1: &str, p2: &str, p3: &str) -> DntkResult {
        match &self.executer.exec(p2) {
            Ok(p4) => {
                DntkResult::Output(self.output_ok(p1, p2, p3, p4)
                                     .ancize()
                                     .to_string())
            },
            Err(e) => {
                match e {
                    bc::BcError::PopenError(e) => panic!("call bc process open error: {:?}", e),
                    bc::BcError::Timeout => panic!("call bc process is timeout"),
                    _ => {
                        DntkResult::Output(self.output_ng(p1, p2, p3)
                                     .ancize()
                                     .to_string())
                    },
                }
            },
        }
    }

    fn dntk_exec(&mut self, ptr: [libc::c_char; 3]) -> DntkResult {
        let mut filtered_vec = vec![ptr[0] as u8, ptr[1] as u8, ptr[2] as u8].into_iter().filter(|p| *p != 0).collect::<Vec<u8>>();
        while !filtered_vec.is_empty() {
            let input_char = &filtered_vec.remove(0);
            match &self.filter_char(*input_char) {
                FilterResult::Unknown(unknown_code) => {
                    self.warning(unknown_code.to_owned());
                },
                FilterResult::Esc => {
                    if ptr.len() >= 3 {
                        match ptr[2] as u8 {
                            util::ASCII_CODE_RIGHT => {
                                self.cursor_move_right();
                                break;
                            },
                            util::ASCII_CODE_LEFT => {
                                self.cursor_move_left();
                                break;
                            },
                            _ => {
                                return DntkResult::Fin
                            }
                        }
                    } else {
                        return DntkResult::Fin
                    }
                },
                FilterResult::End => {
                    return DntkResult::Fin
                },
                FilterResult::Refresh => {
                    self.refresh();
                    return DntkResult::Continue
                },
                FilterResult::Delete => {
                    self.delete_column();
                },
                FilterResult::CurLeft => {
                    self.cursor_move_left();
                },
                FilterResult::CurRight => {
                    self.cursor_move_right();
                },
                FilterResult::Calculatable(code) => {
                    self.insert_column(code.to_owned());
                },
            }
        }
        self.write_stdout(&self.output_fill_whitespace(self.before_printed_len));
        let p1 = util::DNTK_PROMPT.to_string();
        let p2 = &self.statement_from_utf8();
        let p3 = " = ";
        self.calculate(&p1, p2, p3)
    }

    #[cfg(target_os = "windows")]
    fn watch(&self,  mut ptr: [libc::c_char; 3]) -> [libc::c_char; 3] {
        ptr[0] = wconsole::getch(true).unwrap() as u8 as i8;
        ptr
    }

    #[cfg(not(target_os = "windows"))]
    fn watch(&self,  ptr: [libc::c_char; 3]) -> [libc::c_char; 3] {
        loop{
            if unsafe { libc::read(0, ptr.as_ptr() as *mut libc::c_void, 3) } > 0 {
                return ptr
            };
        }
    }

    fn inject_filter2print(&mut self) {
        let p1 = &util::DNTK_PROMPT.to_string();
        let p2 = &mut self.statement_from_utf8();
        let p3 = " = ";
        for i in &self.input_vec {
            match &self.filter_char(i.to_owned()) {
                FilterResult::Calculatable(_) => continue,
                _ => panic!("Injection statement is including unrecoginezed char"),
            }
        }
        if let DntkResult::Output(o) = self.calculate(p1, p2, p3) {
            self.write_stdout(&o);
        }
    }

    fn flush(&self) {
        if std::env::var_os("DNTK_ENV") != Some(std::ffi::OsString::from("TEST")) {
            #[cfg(target_os = "windows")]
            wconsole::set_cursor_visible(false).unwrap();
        }

        std::io::stdout().flush().unwrap();

        if std::env::var_os("DNTK_ENV") != Some(std::ffi::OsString::from("TEST")) {
            #[cfg(target_os = "windows")]
            {
                let vec_cur = wconsole::get_cursor_position().unwrap();
                wconsole::set_cursor_position(util::DNTK_PROMPT.to_string().len() as u16 + self.currnet_cur_pos as u16 -1, vec_cur.y).unwrap();
                wconsole::set_cursor_visible(true).unwrap();
            }
        }
    }

    pub fn run(&mut self) {
        if !atty::is(Stream::Stdin) && std::env::var_os("DNTK_ENV") != Some(std::ffi::OsString::from("TEST")) {
            let mut s = String::new();
            std::io::stdin().read_line(&mut s).ok();
            self.write_stdout_ln(&self.executer.exec(&s).unwrap());
            return
        };

        if util::DNTK_OPT.show_limits {
            self.write_stdout_ln(&self.executer.exec("limits").unwrap());
            return
        }

        self.write_stdout(util::DNTK_PROMPT);
        std::io::stdout().flush().unwrap();

        if util::DNTK_OPT.inject != "" {
            self.inject_filter2print();
            self.flush();

            if util::DNTK_OPT.once {
                self.write_stdout("\n");
                return
            }
        }

        let ptr: [libc::c_char; 3] = [0; 3];
        loop {
            match self.dntk_exec(self.watch(ptr)) {
                DntkResult::Fin => {
                    self.write_stdout("\n");
                    break
                },
                DntkResult::Continue => {
                    continue
                },
                DntkResult::Output(o) => {
                    self.write_stdout(&o);
                },
            }
            self.flush();

            if util::DNTK_OPT.once {
                self.write_stdout("\n");
                return
            }
        }
    }
}