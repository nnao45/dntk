use std::fmt;

use super::util;
use owo_colors::OwoColorize;

#[derive(Debug, PartialEq)]
pub(crate) enum DntkStringType {
    Ok,
    Ng,
    Warn,
    Refresh,
}

#[derive(Debug)]
pub(crate) struct DntkString {
    data: String,
    dtype: DntkStringType,
    #[cfg(not(target_os = "windows"))]
    cur_pos_from_right: usize,
}

impl DntkString {
    pub(crate) fn new(data: String, dtype: DntkStringType, cur_pos_from_right: usize) -> Self {
        Self {
            data,
            dtype,
            #[cfg(not(target_os = "windows"))]
            cur_pos_from_right,
        }
    }

    pub(crate) fn ancize(mut self) -> Self {
        self = self.colorize();
        #[cfg(not(target_os = "windows"))]
        {
            self = self.cursorize();
        }
        self
    }

    pub(crate) fn colorize(mut self) -> Self {
        if !util::DNTK_OPT.white {
            match &self.dtype {
                DntkStringType::Ok => {
                    self.data = self.data.cyan().to_string();
                }
                DntkStringType::Ng => {
                    self.data = self.data.purple().to_string();
                }
                DntkStringType::Warn => {
                    self.data = self.data.yellow().to_string();
                }
                DntkStringType::Refresh => {
                    self.data = self.data.green().to_string();
                }
            }
        }
        self
    }

    #[cfg(not(target_os = "windows"))]
    pub(crate) fn cursorize(mut self) -> Self {
        self.data = format!(
            "{}{}{}{}",
            self.data,
            util::CURSOR_MOVE_ES_HEAD,
            self.cur_pos_from_right,
            util::CURSOR_MOVE_ES_BACK
        );
        self
    }
}

impl fmt::Display for DntkString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.data)
    }
}

#[derive(Debug)]
pub(crate) struct PromptState {
    total_len: usize,
    statement_len: usize,
    result_len: usize,
}

impl Default for PromptState {
    fn default() -> Self {
        Self {
            total_len: util::DNTK_PROMPT.len(),
            statement_len: 0,
            result_len: 0,
        }
    }
}

impl PromptState {
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn whitespace(&self) -> String {
        whitespace(self.total_len)
    }

    pub(crate) fn render_success(
        &mut self,
        prompt: &str,
        statement: &str,
        separator: &str,
        result: &str,
        cursor_pos: usize,
    ) -> DntkString {
        self.statement_len = statement.len();
        self.result_len = result.len();
        self.total_len = prompt.len() + self.statement_len + separator.len() + self.result_len;
        let pos_difference = self.statement_len.saturating_sub(cursor_pos);
        DntkString::new(
            format!("{prompt}{statement}{separator}{result}"),
            DntkStringType::Ok,
            separator.len() + self.result_len + pos_difference,
        )
    }

    pub(crate) fn render_error(
        &mut self,
        prompt: &str,
        statement: &str,
        separator: &str,
        cursor_pos: usize,
    ) -> DntkString {
        self.statement_len = statement.len();
        self.total_len = prompt.len() + self.statement_len + separator.len() + self.result_len;
        let pos_difference = self.statement_len.saturating_sub(cursor_pos);
        DntkString::new(
            format!("{prompt}{statement}{separator}"),
            DntkStringType::Ng,
            separator.len() + pos_difference,
        )
    }
}

pub(crate) fn whitespace(len: usize) -> String {
    format!("\r{}", " ".repeat(len))
}
