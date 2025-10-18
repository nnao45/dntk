use std::collections::BTreeMap;
use std::fmt;

use fasteval::Evaler;
use fasteval::{Parser, Slab};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive, Zero};
use rust_decimal::Decimal;

#[derive(Debug)]
pub enum BcError {
    NoResult,
    /// Parse or evaluation error
    Error(String),
}

impl fmt::Display for BcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BcError::NoResult => write!(f, "No result returned"),
            BcError::Error(msg) => write!(f, "Evaluation error: {}", msg),
        }
    }
}

pub struct BcExecuter {
    parser: Parser,
    slab: Slab,
    namespaces: Vec<BTreeMap<String, f64>>,
}

impl fmt::Debug for BcExecuter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BcExecuter").finish()
    }
}

impl Default for BcExecuter {
    fn default() -> Self {
        BcExecuter {
            parser: Parser::new(),
            slab: Slab::new(),
            namespaces: vec![BTreeMap::new()],
        }
    }
}

impl BcExecuter {
    /// Format the result according to the scale setting
    fn format_result(&self, value: Decimal) -> String {
        let scale = util::DNTK_OPT.scale as u32;

        let rounded = value.round_dp(scale);
        let mut formatted = rounded.to_string();

        if formatted.contains('.') {
            formatted = formatted
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
        }

        if formatted.starts_with("0.") {
            formatted = formatted.trim_start_matches('0').to_string();
        } else if formatted.starts_with("-0.") {
            formatted = format!("-{}", formatted.trim_start_matches("-0"));
        }

        if formatted.is_empty() || formatted == "." || formatted == "-" {
            "0".to_string()
        } else {
            formatted
        }
    }

    pub fn exec(&mut self, statement: &str) -> Result<String, BcError> {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            return Err(BcError::NoResult);
        }
        if trimmed == "limits" {
            return Ok(self.show_limits());
        }

        let processed = self.preprocess_bc_syntax(statement);
        let statements = self.split_statements(&processed);

        let mut last_value: Option<Decimal> = None;
        for stmt in statements {
            if let Some(value) = self.eval_statement(&stmt)? {
                last_value = Some(value);
            }
        }

        let value = last_value.unwrap_or_else(Decimal::zero);
        Ok(self.format_result(value))
    }

    fn eval_statement(&mut self, stmt: &str) -> Result<Option<Decimal>, BcError> {
        let trimmed = stmt.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }

        if trimmed.starts_with("for") {
            return self.eval_for_loop(trimmed);
        }

        if let Some((var, expr)) = Self::detect_assignment(trimmed) {
            if !Self::is_valid_identifier(&var) {
                return Err(BcError::Error(format!("Invalid identifier: {}", var)));
            }
            let assigned = self.eval_assignment(&var, &expr)?;
            return Ok(Some(assigned));
        }

        let value = self.eval_expression(trimmed)?;
        Ok(Some(value))
    }

    fn eval_assignment(&mut self, name: &str, expr: &str) -> Result<Decimal, BcError> {
        let value = self.eval_expression(expr)?;
        let value_f64 = value
            .to_f64()
            .ok_or_else(|| BcError::Error("Failed to convert to f64".to_string()))?;
        if let Some(scope) = self.namespaces.last_mut() {
            scope.insert(name.to_string(), value_f64);
        }
        Ok(value)
    }

    fn eval_expression(&mut self, expr: &str) -> Result<Decimal, BcError> {
        let expr_idx = self
            .parser
            .parse(expr, &mut self.slab.ps)
            .map_err(|e| BcError::Error(e.to_string()))?;
        let expr_ref = expr_idx.from(&self.slab.ps);
        let value = expr_ref
            .eval(&self.slab, &mut self.namespaces)
            .map_err(|e| BcError::Error(e.to_string()))?;

        Decimal::from_f64(value)
            .ok_or_else(|| BcError::Error("Failed to convert to decimal".to_string()))
    }

    fn eval_for_loop(&mut self, stmt: &str) -> Result<Option<Decimal>, BcError> {
        let mut rest = stmt.trim_start();
        rest = rest
            .strip_prefix("for")
            .map(|r| r.trim_start())
            .ok_or_else(|| BcError::Error("Invalid for syntax".to_string()))?;

        if !rest.starts_with('(') {
            return Err(BcError::Error("Expected '(' after for".to_string()));
        }

        let header_end = Self::find_matching(rest, 0, '(', ')')?;
        let header = &rest[1..header_end];
        let after_header = rest[header_end + 1..].trim();

        let parts = Self::split_top_level(header, ';');
        if parts.len() != 3 {
            return Err(BcError::Error(
                "for header must be 'init; condition; post'".to_string(),
            ));
        }

        let init = parts[0].trim();
        let condition = parts[1].trim();
        let post = parts[2].trim();

        if !init.is_empty() {
            self.eval_statement(init)?;
        }

        let (body_statements, remainder) = if after_header.starts_with('{') {
            let block_end = Self::find_matching(after_header, 0, '{', '}')?;
            let content = after_header[1..block_end].to_string();
            let rest = after_header[block_end + 1..].trim();
            (self.split_statements(&content), rest)
        } else if after_header.is_empty() {
            return Err(BcError::Error("for body is missing".to_string()));
        } else {
            (vec![after_header.to_string()], "")
        };

        if !remainder.is_empty() {
            return Err(BcError::Error(
                "Unexpected tokens after for body".to_string(),
            ));
        }

        let mut last_value: Option<Decimal> = None;
        loop {
            let should_continue = if condition.is_empty() {
                true
            } else {
                let cond_value = self.eval_expression(condition)?;
                !cond_value.is_zero()
            };

            if !should_continue {
                break;
            }

            for stmt in &body_statements {
                if let Some(value) = self.eval_statement(stmt)? {
                    last_value = Some(value);
                }
            }

            if !post.is_empty() {
                if let Some(value) = self.eval_statement(post)? {
                    last_value = Some(value);
                }
            }
        }

        Ok(last_value)
    }

    fn split_statements(&self, input: &str) -> Vec<String> {
        let mut statements = Vec::new();
        let mut current = String::new();
        let mut depth_round = 0;
        let mut depth_square = 0;
        let mut depth_curly = 0;

        for ch in input.chars() {
            match ch {
                '(' => depth_round += 1,
                ')' => {
                    if depth_round > 0 {
                        depth_round -= 1;
                    }
                }
                '[' => depth_square += 1,
                ']' => {
                    if depth_square > 0 {
                        depth_square -= 1;
                    }
                }
                '{' => depth_curly += 1,
                '}' => {
                    if depth_curly > 0 {
                        depth_curly -= 1;
                    }
                }
                ';' | '\n' if depth_round == 0 && depth_square == 0 && depth_curly == 0 => {
                    let trimmed = current.trim();
                    if !trimmed.is_empty() {
                        statements.push(trimmed.to_string());
                    }
                    current.clear();
                    continue;
                }
                _ => {}
            }
            current.push(ch);
        }

        let trimmed = current.trim();
        if !trimmed.is_empty() {
            statements.push(trimmed.to_string());
        }

        statements
    }

    fn split_top_level(input: &str, delimiter: char) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut depth_round = 0;
        let mut depth_square = 0;
        let mut depth_curly = 0;

        for ch in input.chars() {
            match ch {
                '(' => depth_round += 1,
                ')' => {
                    if depth_round > 0 {
                        depth_round -= 1;
                    }
                }
                '[' => depth_square += 1,
                ']' => {
                    if depth_square > 0 {
                        depth_square -= 1;
                    }
                }
                '{' => depth_curly += 1,
                '}' => {
                    if depth_curly > 0 {
                        depth_curly -= 1;
                    }
                }
                _ => {}
            }

            if ch == delimiter && depth_round == 0 && depth_square == 0 && depth_curly == 0 {
                parts.push(current.trim().to_string());
                current.clear();
            } else {
                current.push(ch);
            }
        }

        if !current.trim().is_empty() {
            parts.push(current.trim().to_string());
        }

        parts
    }

    fn detect_assignment(stmt: &str) -> Option<(String, String)> {
        let mut depth_round = 0;
        let mut depth_square = 0;
        let mut depth_curly = 0;
        let chars: Vec<char> = stmt.chars().collect();

        for (index, ch) in chars.iter().enumerate() {
            match ch {
                '(' => depth_round += 1,
                ')' => {
                    if depth_round > 0 {
                        depth_round -= 1;
                    }
                }
                '[' => depth_square += 1,
                ']' => {
                    if depth_square > 0 {
                        depth_square -= 1;
                    }
                }
                '{' => depth_curly += 1,
                '}' => {
                    if depth_curly > 0 {
                        depth_curly -= 1;
                    }
                }
                '=' if depth_round == 0 && depth_square == 0 && depth_curly == 0 => {
                    let prev = index.checked_sub(1).and_then(|i| chars.get(i));
                    let next = chars.get(index + 1);
                    if matches!(prev, Some('<') | Some('>') | Some('!')) {
                        continue;
                    }
                    if matches!(next, Some('=')) {
                        continue;
                    }

                    let left = stmt[..index].trim();
                    let right = stmt[index + 1..].trim();
                    if left.is_empty() || right.is_empty() {
                        return None;
                    }
                    return Some((left.to_string(), right.to_string()));
                }
                _ => {}
            }
        }

        None
    }

    fn is_valid_identifier(name: &str) -> bool {
        let mut chars = name.chars();
        match chars.next() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
            _ => return false,
        }
        chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    fn find_matching(input: &str, start: usize, open: char, close: char) -> Result<usize, BcError> {
        let mut depth = 0;
        for (index, ch) in input.char_indices().skip(start) {
            if ch == open {
                depth += 1;
            } else if ch == close {
                depth -= 1;
                if depth == 0 {
                    return Ok(index);
                }
            }
        }
        Err(BcError::Error("Unmatched delimiter".to_string()))
    }

    /// Show limits (bc compatibility)
    fn show_limits(&self) -> String {
        format!(
            "BC_BASE_MAX     = {}\n\
             BC_DIM_MAX      = {}\n\
             BC_SCALE_MAX    = {}\n\
             BC_STRING_MAX   = {}\n\
             MAX Exponent    = {}\n\
             Number of vars  = {}",
            u32::MAX,
            65535,
            i32::MAX,
            i32::MAX,
            1024,
            i32::MAX
        )
    }

    /// Convert bc-specific syntax to standard math syntax
    fn preprocess_bc_syntax(&self, statement: &str) -> String {
        let mut result = statement.to_string();

        result = result.replace("s(", "sin(");
        result = result.replace("c(", "cos(");
        result = result.replace("a(", "atan(");
        result = result.replace("l(", "ln(");
        result = result.replace("e(", "exp(");

        result
    }
}
