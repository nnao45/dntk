use std::collections::HashMap;

use dashu::Decimal;

use super::error::BcError;

#[derive(Debug, Default)]
pub(crate) struct LiteralTable {
    values: HashMap<String, Decimal>,
    counter: usize,
}

impl LiteralTable {
    pub fn reset(&mut self) {
        self.values.clear();
        self.counter = 0;
    }

    pub fn get(&self, name: &str) -> Option<Decimal> {
        self.values.get(name).cloned()
    }

    pub fn substitute(&mut self, expr: &str) -> Result<String, BcError> {
        let mut result = String::with_capacity(expr.len());
        let chars: Vec<char> = expr.chars().collect();
        let mut index = 0;
        while index < chars.len() {
            if let Some((literal, consumed)) = Self::extract_numeric_literal(&chars, index) {
                let name = self.next_literal_name();
                let decimal = literal
                    .parse::<Decimal>()
                    .map_err(|_| BcError::Error(format!("Failed to parse literal: {literal}")))?;
                self.values.insert(name.clone(), decimal);
                result.push_str(&name);
                index += consumed;
            } else {
                result.push(chars[index]);
                index += 1;
            }
        }
        Ok(result)
    }

    fn next_literal_name(&mut self) -> String {
        let name = format!("__dntk_lit{}", self.counter);
        self.counter = self.counter.wrapping_add(1);
        name
    }

    fn extract_numeric_literal(chars: &[char], start: usize) -> Option<(String, usize)> {
        let len = chars.len();
        let mut index = start;
        let mut literal = String::new();

        let prev = Self::previous_non_whitespace(chars, start);

        if index >= len {
            return None;
        }

        if chars[index] == '+' || chars[index] == '-' {
            if !Self::is_unary_literal_start(chars[index], prev) {
                return None;
            }
            literal.push(chars[index]);
            index += 1;
            if index >= len {
                return None;
            }
        }

        let mut has_digits = false;
        let mut has_decimal_point = false;

        if index < len && chars[index].is_ascii_digit() {
            if let Some(p) = prev {
                if p.is_ascii_alphanumeric() || p == '_' {
                    return None;
                }
            }
            has_digits = true;
            while index < len && chars[index].is_ascii_digit() {
                literal.push(chars[index]);
                index += 1;
            }
        }

        if index < len && chars[index] == '.' {
            if let Some(p) = prev {
                if p.is_ascii_alphanumeric() || p == '_' {
                    return None;
                }
            }
            has_decimal_point = true;
            literal.push('.');
            index += 1;
            let mut frac_digits = 0;
            while index < len && chars[index].is_ascii_digit() {
                literal.push(chars[index]);
                index += 1;
                frac_digits += 1;
            }
            has_digits = has_digits || frac_digits > 0;
            if frac_digits == 0 {
                return None;
            }
        } else if !has_digits {
            return None;
        }

        if index < len && (chars[index] == 'e' || chars[index] == 'E') {
            literal.push(chars[index]);
            index += 1;
            if index < len && (chars[index] == '+' || chars[index] == '-') {
                literal.push(chars[index]);
                index += 1;
            }
            let mut exp_digits = 0;
            while index < len && chars[index].is_ascii_digit() {
                literal.push(chars[index]);
                index += 1;
                exp_digits += 1;
            }
            if exp_digits == 0 {
                return None;
            }
        }

        if !has_digits {
            return None;
        }

        if index < len {
            let next = chars[index];
            if next.is_ascii_alphanumeric() || next == '_' {
                return None;
            }
            if next == '.'
                && !has_decimal_point
                && index + 1 < len
                && chars[index + 1].is_ascii_alphabetic()
            {
                return None;
            }
        }

        Some((literal, index - start))
    }

    fn previous_non_whitespace(chars: &[char], index: usize) -> Option<char> {
        if index == 0 {
            return None;
        }
        let mut pos = index;
        while pos > 0 {
            pos -= 1;
            let ch = chars[pos];
            if !ch.is_whitespace() {
                return Some(ch);
            }
        }
        None
    }

    fn is_unary_literal_start(current: char, prev: Option<char>) -> bool {
        if let Some(p) = prev {
            if p.is_ascii_alphanumeric() || p == '_' || p == ')' {
                return false;
            }
        }
        current == '+' || current == '-'
    }
}
