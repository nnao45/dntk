use super::buffer::InputBuffer;
use super::history::History;
use super::prompt::{self, DntkString, DntkStringType, PromptState};
use super::util;
use crate::dntker::bc;
use std::io::Write;
use std::io::{stdout, BufWriter, IsTerminal};

#[cfg(target_os = "windows")]
use winconsole::console as wconsole;

#[derive(Debug)]
pub struct Dntker {
    executer: bc::BcExecuter,
    buffer: InputBuffer,
    prompt: PromptState,
    history: History,
}

impl Default for Dntker {
    fn default() -> Self {
        let buffer = if util::DNTK_OPT.inject.is_empty() {
            InputBuffer::new()
        } else {
            InputBuffer::with_inject(&util::DNTK_OPT.inject)
        };

        Dntker {
            executer: Default::default(),
            buffer,
            prompt: PromptState::default(),
            history: History::load(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum FilterResult {
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

impl Dntker {
    #[allow(unused_must_use)]
    fn write_stdout(&self, buf: &str) {
        let out = stdout();
        let mut out = BufWriter::new(out.lock());
        write!(out, "{buf}");
    }

    fn write_stdout_ln(&self, buf: &str) {
        self.write_stdout(&format!("{buf}\n"))
    }

    pub(crate) fn filter_char(&self, ascii_char: u8) -> FilterResult {
        match ascii_char {
            util::ASCII_CODE_ZERO => FilterResult::Calculatable(util::ASCII_CODE_ZERO),
            util::ASCII_CODE_ONE => FilterResult::Calculatable(util::ASCII_CODE_ONE),
            util::ASCII_CODE_TWO => FilterResult::Calculatable(util::ASCII_CODE_TWO),
            util::ASCII_CODE_THREE => FilterResult::Calculatable(util::ASCII_CODE_THREE),
            util::ASCII_CODE_FOUR => FilterResult::Calculatable(util::ASCII_CODE_FOUR),
            util::ASCII_CODE_FIVE => FilterResult::Calculatable(util::ASCII_CODE_FIVE),
            util::ASCII_CODE_SIX => FilterResult::Calculatable(util::ASCII_CODE_SIX),
            util::ASCII_CODE_SEVEN => FilterResult::Calculatable(util::ASCII_CODE_SEVEN),
            util::ASCII_CODE_EIGHT => FilterResult::Calculatable(util::ASCII_CODE_EIGHT),
            util::ASCII_CODE_NINE => FilterResult::Calculatable(util::ASCII_CODE_NINE),
            util::ASCII_CODE_ROUNDLEFT => FilterResult::Calculatable(util::ASCII_CODE_ROUNDLEFT),
            util::ASCII_CODE_ROUNDRIGHT => FilterResult::Calculatable(util::ASCII_CODE_ROUNDRIGHT),
            util::ASCII_CODE_SQUARELEFT => FilterResult::CurLeft,
            util::ASCII_CODE_SQUARERIGHT => FilterResult::CurRight,
            util::ASCII_CODE_BACKSLASH => FilterResult::Calculatable(util::ASCII_CODE_BACKSLASH),
            util::ASCII_CODE_CURLYLEFT => FilterResult::Calculatable(util::ASCII_CODE_CURLYLEFT),
            util::ASCII_CODE_CURLYRIGHT => FilterResult::Calculatable(util::ASCII_CODE_CURLYRIGHT),
            util::ASCII_CODE_LARGER => FilterResult::Calculatable(util::ASCII_CODE_LARGER),
            util::ASCII_CODE_SMALLER => FilterResult::Calculatable(util::ASCII_CODE_SMALLER),
            util::ASCII_CODE_PLUS => FilterResult::Calculatable(util::ASCII_CODE_PLUS),
            util::ASCII_CODE_MINUS => FilterResult::Calculatable(util::ASCII_CODE_MINUS),
            util::ASCII_CODE_ASTERISK => FilterResult::Calculatable(util::ASCII_CODE_ASTERISK),
            util::ASCII_CODE_SLUSH => FilterResult::Calculatable(util::ASCII_CODE_SLUSH),
            util::ASCII_CODE_HAT => FilterResult::Calculatable(util::ASCII_CODE_HAT),
            util::ASCII_CODE_PERCENT => FilterResult::Calculatable(util::ASCII_CODE_PERCENT),
            util::ASCII_CODE_DOT => FilterResult::Calculatable(util::ASCII_CODE_DOT),
            util::ASCII_CODE_COMMA => FilterResult::Calculatable(util::ASCII_CODE_COMMA),
            util::ASCII_CODE_BIKKURI => FilterResult::Calculatable(util::ASCII_CODE_BIKKURI),
            util::ASCII_CODE_EQUAL => FilterResult::Calculatable(util::ASCII_CODE_EQUAL),
            util::ASCII_CODE_PIPE => FilterResult::Calculatable(util::ASCII_CODE_PIPE),
            util::ASCII_CODE_AND => FilterResult::Calculatable(util::ASCII_CODE_AND),
            util::ASCII_CODE_SEMICOLON => FilterResult::Calculatable(util::ASCII_CODE_SEMICOLON),
            util::ASCII_CODE_UNDERSCORE => FilterResult::Calculatable(util::ASCII_CODE_UNDERSCORE),
            b'A'..=b'Z' => FilterResult::Calculatable(ascii_char),
            b'a'..=b'z' => FilterResult::Calculatable(ascii_char),
            util::ASCII_CODE_AT => FilterResult::Refresh,
            util::ASCII_CODE_WINENTER => FilterResult::End,
            util::ASCII_CODE_NEWLINE => FilterResult::End,
            util::ASCII_CODE_ESCAPE => FilterResult::Esc,
            util::ASCII_CODE_BACKSPACE => FilterResult::Delete,
            util::ASCII_CODE_DELETE => FilterResult::Delete,
            util::ASCII_CODE_SPACE => FilterResult::Calculatable(util::ASCII_CODE_SPACE),
            unknown_code => FilterResult::Unknown(unknown_code),
        }
    }

    pub(crate) fn delete_column(&mut self) {
        if self.buffer.delete() {
            self.history.reset_navigation();
        }
    }

    pub(crate) fn cursor_move_left(&mut self) {
        self.buffer.move_left();
    }

    pub(crate) fn cursor_move_right(&mut self) {
        self.buffer.move_right();
    }

    pub(crate) fn insert_column(&mut self, code: u8) {
        self.buffer.insert(code);
        self.history.reset_navigation();
    }

    fn replace_input_with(&mut self, statement: String) {
        self.write_stdout(&self.prompt.whitespace());
        self.buffer.replace(&statement);

        if self.buffer.is_empty() {
            self.prompt.reset();
            self.write_stdout(util::DNTK_PROMPT);
            std::io::stdout().flush().unwrap();
            return;
        }

        let prompt = util::DNTK_PROMPT.to_string();
        let separator = " = ";
        if let DntkResult::Output(output) = self.calculate(&prompt, &statement, separator) {
            self.write_stdout(&output);
            self.flush();
        }
    }

    fn recall_history_previous(&mut self) -> bool {
        if let Some(entry) = self.history.previous() {
            self.replace_input_with(entry);
            return true;
        }
        false
    }

    fn recall_history_next(&mut self) -> bool {
        if let Some(entry) = self.history.next() {
            self.replace_input_with(entry);
            return true;
        }
        false
    }

    pub(crate) fn statement_from_utf8(&self) -> String {
        self.buffer.statement()
    }

    pub(crate) fn cursor(&self) -> usize {
        self.buffer.cursor()
    }

    fn inform(&mut self, msg: &str, dtype: DntkStringType) {
        if util::DNTK_OPT.quiet
            || std::env::var_os("DNTK_ENV") == Some(std::ffi::OsString::from("TEST"))
        {
            return;
        }

        self.write_stdout(&self.prompt.whitespace());
        self.write_stdout(
            DntkString::new(format!("\r{msg}"), dtype, 0)
                .colorize()
                .to_string()
                .as_ref(),
        );
        std::io::stdout().flush().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1000));
        self.write_stdout(&prompt::whitespace(msg.len()));
    }

    fn warning(&mut self, unknown_code: u8) {
        let ch = unknown_code as char;
        self.inform(
            &format!("this char is no supported: {ch}"),
            DntkStringType::Warn,
        )
    }

    pub(crate) fn refresh(&mut self) {
        self.inform("refresh!!", DntkStringType::Refresh);
        self.write_stdout(&self.prompt.whitespace());
        let dnew: Dntker = Default::default();
        *self = dnew;
        self.write_stdout(util::DNTK_PROMPT);
        std::io::stdout().flush().unwrap();
    }

    fn calculate(&mut self, prompt: &str, statement: &str, separator: &str) -> DntkResult {
        match self.executer.exec(statement) {
            Ok(result) => {
                let rendered = self
                    .prompt
                    .render_success(prompt, statement, separator, &result, self.cursor())
                    .ancize()
                    .to_string();
                DntkResult::Output(rendered)
            }
            Err(_e) => DntkResult::Output(
                self.prompt
                    .render_error(prompt, statement, separator, self.cursor())
                    .ancize()
                    .to_string(),
            ),
        }
    }

    fn dntk_exec(&mut self, ptr: [libc::c_char; 3]) -> DntkResult {
        let mut filtered_vec = vec![ptr[0] as u8, ptr[1] as u8, ptr[2] as u8]
            .into_iter()
            .filter(|p| *p != 0)
            .collect::<Vec<u8>>();
        while !filtered_vec.is_empty() {
            let input_char = &filtered_vec.remove(0);
            match &self.filter_char(*input_char) {
                FilterResult::Unknown(unknown_code) => {
                    self.warning(*unknown_code);
                }
                FilterResult::Esc => {
                    if ptr.len() >= 3 {
                        match ptr[2] as u8 {
                            util::ASCII_CODE_RIGHT => {
                                self.cursor_move_right();
                                break;
                            }
                            util::ASCII_CODE_LEFT => {
                                self.cursor_move_left();
                                break;
                            }
                            util::ASCII_CODE_UP => {
                                if self.recall_history_previous() {
                                    return DntkResult::Continue;
                                }
                                break;
                            }
                            util::ASCII_CODE_DOWN => {
                                if self.recall_history_next() {
                                    return DntkResult::Continue;
                                }
                                break;
                            }
                            _ => {
                                return DntkResult::Fin;
                            }
                        }
                    } else {
                        return DntkResult::Fin;
                    }
                }
                FilterResult::End => {
                    if self.cursor() > 0 && self.buffer.last() == Some(b'\\') {
                        self.buffer.pop_last();
                        self.buffer.push(b' ');
                        self.history.reset_navigation();
                        let statement = self.statement_from_utf8();
                        self.replace_input_with(statement);
                        return DntkResult::Continue;
                    }
                    let statement = self.statement_from_utf8();
                    self.history.push(&statement);
                    self.write_stdout(&self.prompt.whitespace());

                    if !statement.trim().is_empty() {
                        let prompt = util::DNTK_PROMPT.to_string();
                        let separator = " = ";
                        if let DntkResult::Output(output) =
                            self.calculate(&prompt, &statement, separator)
                        {
                            self.write_stdout(&output);
                        }
                    }

                    self.write_stdout("\n");
                    if util::DNTK_OPT.once {
                        self.flush();
                        return DntkResult::Fin;
                    }

                    self.prompt.reset();
                    self.buffer.replace("");
                    self.write_stdout(util::DNTK_PROMPT);
                    self.flush();

                    return DntkResult::Continue;
                }
                FilterResult::Refresh => {
                    self.refresh();
                    return DntkResult::Continue;
                }
                FilterResult::Delete => {
                    self.delete_column();
                }
                FilterResult::CurLeft => {
                    self.cursor_move_left();
                }
                FilterResult::CurRight => {
                    self.cursor_move_right();
                }
                FilterResult::Calculatable(code) => {
                    self.insert_column(*code);
                }
            }
        }
        self.write_stdout(&self.prompt.whitespace());
        let prompt = util::DNTK_PROMPT.to_string();
        let statement = self.statement_from_utf8();
        let separator = " = ";
        self.calculate(&prompt, &statement, separator)
    }

    #[cfg(target_os = "windows")]
    fn watch(&self, mut ptr: [libc::c_char; 3]) -> [libc::c_char; 3] {
        ptr[0] = wconsole::getch(true).unwrap() as u8 as i8;
        ptr
    }

    #[cfg(not(target_os = "windows"))]
    fn watch(&self, ptr: [libc::c_char; 3]) -> [libc::c_char; 3] {
        loop {
            if unsafe { libc::read(0, ptr.as_ptr() as *mut libc::c_void, 3) } > 0 {
                return ptr;
            };
        }
    }

    fn inject_filter2print(&mut self) {
        let prompt = util::DNTK_PROMPT.to_string();
        let statement = self.statement_from_utf8();
        let separator = " = ";
        for i in self.buffer.as_bytes() {
            match &self.filter_char(*i) {
                FilterResult::Calculatable(_) => continue,
                _ => panic!("Injection statement is including unrecoginezed char"),
            }
        }
        if let DntkResult::Output(o) = self.calculate(&prompt, &statement, separator) {
            self.write_stdout(&o);
        }
        self.history.push(&statement);
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
                let x = util::DNTK_PROMPT.len() as u16 + self.cursor().saturating_sub(1) as u16;
                wconsole::set_cursor_position(x, vec_cur.y).unwrap();
                wconsole::set_cursor_visible(true).unwrap();
            }
        }
    }

    pub fn run(&mut self) {
        if !std::io::stdin().is_terminal()
            && std::env::var_os("DNTK_ENV") != Some(std::ffi::OsString::from("TEST"))
        {
            let mut s = String::new();
            std::io::stdin().read_line(&mut s).ok();
            let output = self.executer.exec(&s).unwrap();
            self.write_stdout_ln(&output);
            return;
        };

        if util::DNTK_OPT.show_limits {
            let limits = self.executer.exec("limits").unwrap();
            self.write_stdout_ln(&limits);
            return;
        }

        self.write_stdout(util::DNTK_PROMPT);
        std::io::stdout().flush().unwrap();

        if !util::DNTK_OPT.inject.is_empty() {
            self.inject_filter2print();
            self.flush();

            if util::DNTK_OPT.once {
                self.write_stdout("\n");
                return;
            }
        }

        let ptr: [libc::c_char; 3] = [0; 3];
        loop {
            match self.dntk_exec(self.watch(ptr)) {
                DntkResult::Fin => {
                    self.write_stdout("\n");
                    break;
                }
                DntkResult::Continue => {
                    continue;
                }
                DntkResult::Output(o) => {
                    self.write_stdout(&o);
                }
            }
            self.flush();

            if util::DNTK_OPT.once {
                self.write_stdout("\n");
                return;
            }
        }
    }
}
