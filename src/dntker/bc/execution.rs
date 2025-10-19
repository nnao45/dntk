use std::fmt;
use std::str::FromStr;

use dashu::base::{Abs, Sign};
use dashu::Decimal;
use fasteval::Parser;
use num_traits::{ToPrimitive, Zero};

use super::complex::ComplexNumber;
use super::error::BcError;
use super::literals::LiteralTable;
use super::runtime::{FunctionDef, Runtime, StatementOutcome};
use super::util;

pub struct BcExecuter {
    pub(crate) parser: Parser,
    pub(crate) runtime: Runtime,
    pub(crate) literals: LiteralTable,
}

impl fmt::Debug for BcExecuter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BcExecuter").finish()
    }
}

impl Default for BcExecuter {
    fn default() -> Self {
        let scale = util::DNTK_OPT.scale as u32;
        BcExecuter {
            parser: Parser::new(),
            runtime: Runtime::with_defaults(scale),
            literals: LiteralTable::default(),
        }
    }
}

impl BcExecuter {
    pub fn exec(&mut self, statement: &str) -> Result<String, BcError> {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            return Err(BcError::NoResult);
        }
        if trimmed == "limits" {
            return Ok(self.show_limits());
        }
        if let Some(result) = self.try_eval_complex_expression(trimmed)? {
            return Ok(result);
        }
        if let Some(result) = self.try_eval_matrix_expression(trimmed)? {
            return Ok(result);
        }

        let statements = self.split_statements(trimmed);
        let mut last_value: Option<Decimal> = None;

        for stmt in statements {
            match self.eval_statement(stmt)? {
                StatementOutcome::Return(value) => {
                    last_value = Some(value);
                    break;
                }
                StatementOutcome::Value(value) => {
                    last_value = Some(value);
                }
                StatementOutcome::None => {}
            }
        }

        let value = last_value.unwrap_or(Decimal::ZERO);
        Ok(self.format_result(value))
    }

    fn eval_statement(&mut self, stmt: &str) -> Result<StatementOutcome, BcError> {
        let trimmed = stmt.trim();
        if trimmed.is_empty() {
            return Ok(StatementOutcome::None);
        }

        if Self::starts_with_keyword(trimmed, "define") {
            let remainder = self.define_function(trimmed)?;
            if let Some(rest) = remainder {
                let mut last = StatementOutcome::None;
                for extra in self.split_statements(&rest) {
                    let outcome = self.eval_statement(extra)?;
                    match outcome {
                        StatementOutcome::Return(_) => return Ok(outcome),
                        StatementOutcome::Value(value) => {
                            last = StatementOutcome::value(value);
                        }
                        StatementOutcome::None => {}
                    }
                }
                return Ok(last);
            }
            return Ok(StatementOutcome::None);
        }
        if Self::starts_with_keyword(trimmed, "return") {
            return self.eval_return(trimmed);
        }
        if Self::starts_with_keyword(trimmed, "if") {
            return self.eval_if(trimmed);
        }
        if Self::starts_with_keyword(trimmed, "while") {
            return self.eval_while(trimmed);
        }
        if Self::starts_with_keyword(trimmed, "for") {
            return self.eval_for_loop(trimmed);
        }

        if let Some((name, expr)) = Self::detect_assignment(trimmed) {
            let value = self.eval_assignment(name, expr)?;
            return Ok(StatementOutcome::value(value));
        }

        let value = self.eval_expression(trimmed)?;
        Ok(StatementOutcome::value(value))
    }

    fn eval_if(&mut self, stmt: &str) -> Result<StatementOutcome, BcError> {
        let mut rest = stmt.trim_start();
        rest = rest["if".len()..].trim_start();
        if !rest.starts_with('(') {
            return Err(BcError::Error("Expected '(' after if".to_string()));
        }
        let condition_end = Self::find_matching(rest, 0, '(', ')')?;
        let condition_expr = &rest[1..condition_end];
        let mut remainder = rest[condition_end + 1..].trim_start();

        let condition_value = self.eval_expression(condition_expr)?;
        let condition_true = !condition_value.is_zero();

        let (then_branch, after_then) = self.parse_branch(remainder)?;
        remainder = after_then.trim_start();

        let mut else_branch: Vec<&str> = Vec::new();
        if remainder.starts_with("else") {
            remainder = remainder["else".len()..].trim_start();
            let (branch, after_else) = self.parse_branch(remainder)?;
            else_branch = branch;
            remainder = after_else.trim_start();
        }

        if !remainder.is_empty() {
            return Err(BcError::Error(
                "Unexpected tokens after if statement".to_string(),
            ));
        }

        if condition_true {
            return self.eval_block(then_branch.iter().copied());
        }
        if !else_branch.is_empty() {
            return self.eval_block(else_branch.iter().copied());
        }

        Ok(StatementOutcome::None)
    }

    fn eval_while(&mut self, stmt: &str) -> Result<StatementOutcome, BcError> {
        let mut rest = stmt.trim_start();
        rest = rest["while".len()..].trim_start();
        if !rest.starts_with('(') {
            return Err(BcError::Error("Expected '(' after while".to_string()));
        }

        let condition_end = Self::find_matching(rest, 0, '(', ')')?;
        let condition_expr = &rest[1..condition_end];
        let remainder = rest[condition_end + 1..].trim_start();
        let (body, after_body) = self.parse_branch(remainder)?;

        if !after_body.trim().is_empty() {
            return Err(BcError::Error(
                "Unexpected tokens after while body".to_string(),
            ));
        }

        let mut last_value = StatementOutcome::None;

        loop {
            let cond_value = self.eval_expression(condition_expr)?;
            if cond_value.is_zero() {
                break;
            }

            match self.eval_block(body.iter().copied())? {
                StatementOutcome::Return(value) => return Ok(StatementOutcome::Return(value)),
                StatementOutcome::Value(value) => {
                    last_value = StatementOutcome::value(value);
                }
                StatementOutcome::None => {}
            }
        }

        Ok(last_value)
    }

    fn eval_for_loop(&mut self, stmt: &str) -> Result<StatementOutcome, BcError> {
        let mut rest = stmt.trim_start();
        rest = rest["for".len()..].trim_start();
        if !rest.starts_with('(') {
            return Err(BcError::Error("Expected '(' after for".to_string()));
        }

        let header_end = Self::find_matching(rest, 0, '(', ')')?;
        let header = &rest[1..header_end];
        let after_header = rest[header_end + 1..].trim_start();

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

        let (body, remainder) = self.parse_branch(after_header)?;
        if !remainder.trim().is_empty() {
            return Err(BcError::Error(
                "Unexpected tokens after for body".to_string(),
            ));
        }

        let mut last_value = StatementOutcome::None;
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

            match self.eval_block(body.iter().copied())? {
                StatementOutcome::Return(value) => {
                    return Ok(StatementOutcome::Return(value));
                }
                StatementOutcome::Value(value) => {
                    last_value = StatementOutcome::value(value);
                }
                StatementOutcome::None => {}
            }

            if !post.is_empty() {
                match self.eval_statement(post)? {
                    StatementOutcome::Return(value) => {
                        return Ok(StatementOutcome::Return(value));
                    }
                    StatementOutcome::Value(value) => {
                        last_value = StatementOutcome::value(value);
                    }
                    StatementOutcome::None => {}
                }
            }
        }

        Ok(last_value)
    }

    pub(super) fn eval_block<'a, I>(&mut self, statements: I) -> Result<StatementOutcome, BcError>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut last_value = StatementOutcome::None;
        for stmt in statements {
            match self.eval_statement(stmt)? {
                StatementOutcome::Return(value) => {
                    return Ok(StatementOutcome::Return(value));
                }
                StatementOutcome::Value(value) => {
                    last_value = StatementOutcome::value(value);
                }
                StatementOutcome::None => {}
            }
        }
        Ok(last_value)
    }

    fn eval_return(&mut self, stmt: &str) -> Result<StatementOutcome, BcError> {
        let rest = stmt.trim_start()["return".len()..].trim_start();
        if rest.is_empty() {
            return Ok(StatementOutcome::ret(Decimal::ZERO));
        }
        let expression = if rest.starts_with('(') && rest.ends_with(')') {
            &rest[1..rest.len() - 1]
        } else {
            rest
        };
        let value = self.eval_expression(expression)?;
        Ok(StatementOutcome::ret(value))
    }

    fn define_function(&mut self, stmt: &str) -> Result<Option<String>, BcError> {
        let mut rest = stmt.trim_start();
        rest = rest["define".len()..].trim_start();

        let name_end = rest
            .find(|c: char| c == '(' || c.is_whitespace())
            .ok_or_else(|| BcError::Error("Invalid function definition".to_string()))?;
        let name = rest[..name_end].trim();
        if name.is_empty() {
            return Err(BcError::Error("Function name is required".to_string()));
        }

        rest = rest[name_end..].trim_start();
        if !rest.starts_with('(') {
            return Err(BcError::Error(
                "Expected '(' in function definition".to_string(),
            ));
        }
        let params_end = Self::find_matching(rest, 0, '(', ')')?;
        let params_str = &rest[1..params_end];
        let params: Vec<String> = if params_str.trim().is_empty() {
            Vec::new()
        } else {
            params_str
                .split(',')
                .map(|p| p.trim().to_string())
                .collect()
        };
        rest = rest[params_end + 1..].trim_start();

        if !rest.starts_with('{') {
            return Err(BcError::Error(
                "Expected '{' to start function body".to_string(),
            ));
        }
        let body_end = Self::find_matching(rest, 0, '{', '}')?;
        let body_content = &rest[1..body_end];
        let mut remainder = rest[body_end + 1..].trim_start();
        while remainder.starts_with(';') {
            remainder = remainder[1..].trim_start();
        }
        let body: Vec<String> = self
            .split_statements(body_content)
            .into_iter()
            .map(|stmt| stmt.to_string())
            .collect();

        self.runtime.define_function(
            name.to_string(),
            FunctionDef {
                params: params.into(),
                body: body.into(),
            },
        );
        if remainder.is_empty() {
            Ok(None)
        } else {
            Ok(Some(remainder.to_string()))
        }
    }

    fn eval_assignment(&mut self, name: &str, expr: &str) -> Result<Decimal, BcError> {
        if !Self::is_valid_identifier(name) {
            return Err(BcError::Error(format!("Invalid identifier: {name}")));
        }

        let value = self.eval_expression(expr)?;
        if self.apply_special_assignment(name, &value)? {
            return Ok(value);
        }

        if let Some(scope) = self.runtime.find_scope_mut(name) {
            scope.insert(name.to_string(), value.clone());
        } else if let Some(scope) = self.runtime.current_scope_mut() {
            scope.insert(name.to_string(), value.clone());
        }

        Ok(value)
    }

    fn apply_special_assignment(&mut self, name: &str, value: &Decimal) -> Result<bool, BcError> {
        match name {
            "scale" => {
                if value.sign() == Sign::Negative {
                    return Err(BcError::Error("scale() must be non-negative".to_string()));
                }
                let new_scale = ToPrimitive::to_u32(&value.trunc())
                    .ok_or_else(|| BcError::Error("scale() out of range".to_string()))?;
                self.runtime.set_scale(new_scale);
                Ok(true)
            }
            "obase" => {
                if value.sign() == Sign::Negative {
                    return Err(BcError::Error("obase must be positive".to_string()));
                }
                let new_obase = ToPrimitive::to_u32(&value.trunc())
                    .ok_or_else(|| BcError::Error("obase out of range".to_string()))?;
                if !(2..=36).contains(&new_obase) {
                    return Err(BcError::Error("obase must be between 2 and 36".to_string()));
                }
                self.runtime.set_obase(new_obase);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(super) fn format_complex_result(&self, real: Decimal, imag: Decimal) -> String {
        if imag.is_zero() {
            return self.format_result_decimal(&real);
        }
        let imag_abs = imag.clone().abs();
        let imag_str = self.format_result_decimal(&imag_abs);
        if real.is_zero() {
            if imag.sign() == Sign::Negative {
                format!("-{imag_str}i")
            } else {
                format!("{imag_str}i")
            }
        } else {
            let real_str = self.format_result_decimal(&real);
            let sign = if imag.sign() == Sign::Negative {
                "-"
            } else {
                "+"
            };
            format!("{real_str} {sign} {imag_str}i")
        }
    }

    pub(super) fn format_matrix(&self, matrix: &[Vec<ComplexNumber>]) -> String {
        let mut rows = Vec::with_capacity(matrix.len());
        for row in matrix {
            let formatted: Vec<String> = row
                .iter()
                .map(|value| {
                    if value.imag.is_zero() {
                        self.format_result_decimal(&value.real)
                    } else {
                        self.format_complex_result(value.real.clone(), value.imag.clone())
                    }
                })
                .collect();
            rows.push(format!("[{}]", formatted.join(", ")));
        }
        format!("[{}]", rows.join("; "))
    }

    pub(super) fn extract_matrix_literal(input: &str) -> Result<(&str, &str), BcError> {
        let trimmed = input.trim_start();
        if !trimmed.starts_with('[') {
            return Err(BcError::Error(
                "matrix literal must start with '['".to_string(),
            ));
        }
        let mut depth = 0_i32;
        let mut end_index = None;
        for (idx, ch) in trimmed.char_indices() {
            match ch {
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        end_index = Some(idx);
                        break;
                    }
                }
                _ => {}
            }
        }
        let end = end_index
            .ok_or_else(|| BcError::Error("matrix literal has unbalanced brackets".to_string()))?;
        let literal = &trimmed[..=end];
        let remainder = trimmed[end + 1..].trim_start();
        Ok((literal, remainder))
    }

    pub(super) fn parse_matrix(&self, literal: &str) -> Result<Vec<Vec<ComplexNumber>>, BcError> {
        let trimmed = literal.trim();
        if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
            return Err(BcError::Error(
                "matrix literal must be an array".to_string(),
            ));
        }
        let mut rows = Vec::new();
        let mut index = 1usize;
        let len = trimmed.len();
        let mut expected_len: Option<usize> = None;
        while index < len - 1 {
            index = Self::skip_whitespace(trimmed, index);
            if index >= len - 1 {
                break;
            }
            if !trimmed[index..].starts_with('[') {
                return Err(BcError::Error(
                    "matrix must be an array of arrays".to_string(),
                ));
            }
            let (row, next_index) = self.parse_matrix_row(trimmed, index)?;
            if row.is_empty() {
                return Err(BcError::Error(
                    "matrix rows must contain at least one element".to_string(),
                ));
            }
            if let Some(expected) = expected_len {
                if row.len() != expected {
                    return Err(BcError::Error(
                        "matrix rows must have consistent length".to_string(),
                    ));
                }
            } else {
                expected_len = Some(row.len());
            }
            rows.push(row);
            index = Self::skip_whitespace(trimmed, next_index);
            if index >= len - 1 {
                break;
            }
            match trimmed.as_bytes()[index] {
                b',' | b';' => index += 1,
                b']' => break,
                _ => {
                    return Err(BcError::Error(
                        "matrix literal has invalid separators".to_string(),
                    ));
                }
            }
        }
        if rows.is_empty() {
            return Err(BcError::Error(
                "matrix must contain at least one row".to_string(),
            ));
        }
        Ok(rows)
    }

    fn parse_matrix_row(
        &self,
        literal: &str,
        start: usize,
    ) -> Result<(Vec<ComplexNumber>, usize), BcError> {
        let mut index = start;
        if !literal[index..].starts_with('[') {
            return Err(BcError::Error(
                "matrix must be an array of arrays".to_string(),
            ));
        }
        index += 1;
        let mut entries = Vec::new();
        loop {
            index = Self::skip_whitespace(literal, index);
            if index >= literal.len() {
                return Err(BcError::Error(
                    "matrix literal has unbalanced brackets".to_string(),
                ));
            }
            if literal.as_bytes()[index] == b']' {
                index += 1;
                break;
            }
            let (entry, next_index) = Self::extract_matrix_entry(literal, index)?;
            let value = self.parse_matrix_entry(entry)?;
            entries.push(value);
            index = Self::skip_whitespace(literal, next_index);
            if index >= literal.len() {
                return Err(BcError::Error(
                    "matrix literal has unbalanced brackets".to_string(),
                ));
            }
            match literal.as_bytes()[index] {
                b',' => index += 1,
                b']' => {
                    index += 1;
                    break;
                }
                _ => {
                    return Err(BcError::Error(
                        "matrix literal has invalid separators".to_string(),
                    ));
                }
            }
        }
        Ok((entries, index))
    }

    fn extract_matrix_entry(literal: &str, start: usize) -> Result<(&str, usize), BcError> {
        let mut depth = 0i32;
        for (offset, ch) in literal[start..].char_indices() {
            match ch {
                '[' => {
                    return Err(BcError::Error(
                        "matrix entries must be scalar values".to_string(),
                    ));
                }
                '(' => depth += 1,
                ')' => {
                    if depth == 0 {
                        return Err(BcError::Error(
                            "matrix entry has mismatched parentheses".to_string(),
                        ));
                    }
                    depth -= 1;
                }
                ',' | ']' if depth == 0 => {
                    let end = start + offset;
                    let entry = literal[start..end].trim();
                    if entry.is_empty() {
                        return Err(BcError::Error("matrix entry cannot be empty".to_string()));
                    }
                    return Ok((entry, end));
                }
                _ => {}
            }
        }
        if depth != 0 {
            return Err(BcError::Error(
                "matrix entry has mismatched parentheses".to_string(),
            ));
        }
        let entry = literal[start..].trim();
        if entry.is_empty() {
            return Err(BcError::Error("matrix entry cannot be empty".to_string()));
        }
        Ok((entry, literal.len()))
    }

    fn parse_matrix_entry(&self, entry: &str) -> Result<ComplexNumber, BcError> {
        match self.parse_complex_literal(entry) {
            Ok(Some(value)) => Ok(value),
            Ok(None) => {
                let decimal = Decimal::from_str(entry)
                    .map_err(|_| BcError::Error("matrix entry is not a number".to_string()))?;
                Ok(ComplexNumber::from_real(decimal))
            }
            Err(error) => Err(BcError::Error(error.into_message())),
        }
    }

    fn skip_whitespace(input: &str, mut index: usize) -> usize {
        while index < input.len() {
            let ch = input[index..].chars().next().unwrap();
            if ch.is_whitespace() {
                index += ch.len_utf8();
            } else {
                break;
            }
        }
        index
    }

    pub(super) fn matrix_add(
        lhs: &[Vec<ComplexNumber>],
        rhs: &[Vec<ComplexNumber>],
    ) -> Result<Vec<Vec<ComplexNumber>>, BcError> {
        if lhs.len() != rhs.len() || lhs.first().map(|r| r.len()) != rhs.first().map(|r| r.len()) {
            return Err(BcError::Error(
                "matrix addition requires matrices of the same shape".to_string(),
            ));
        }
        let mut result = Vec::with_capacity(lhs.len());
        for (lhs_row, rhs_row) in lhs.iter().zip(rhs.iter()) {
            let mut row = Vec::with_capacity(lhs_row.len());
            for (lhs_value, rhs_value) in lhs_row.iter().zip(rhs_row.iter()) {
                row.push(lhs_value.add(rhs_value));
            }
            result.push(row);
        }
        Ok(result)
    }

    pub(super) fn matrix_sub(
        lhs: &[Vec<ComplexNumber>],
        rhs: &[Vec<ComplexNumber>],
    ) -> Result<Vec<Vec<ComplexNumber>>, BcError> {
        if lhs.len() != rhs.len() || lhs.first().map(|r| r.len()) != rhs.first().map(|r| r.len()) {
            return Err(BcError::Error(
                "matrix subtraction requires matrices of the same shape".to_string(),
            ));
        }
        let mut result = Vec::with_capacity(lhs.len());
        for (lhs_row, rhs_row) in lhs.iter().zip(rhs.iter()) {
            let mut row = Vec::with_capacity(lhs_row.len());
            for (lhs_value, rhs_value) in lhs_row.iter().zip(rhs_row.iter()) {
                row.push(lhs_value.sub(rhs_value));
            }
            result.push(row);
        }
        Ok(result)
    }

    pub(super) fn matrix_mul(
        lhs: &[Vec<ComplexNumber>],
        rhs: &[Vec<ComplexNumber>],
    ) -> Result<Vec<Vec<ComplexNumber>>, BcError> {
        if lhs.is_empty() || rhs.is_empty() {
            return Err(BcError::Error(
                "matrix multiplication requires non-empty matrices".to_string(),
            ));
        }
        let lhs_cols = lhs[0].len();
        if lhs_cols == 0 || rhs[0].is_empty() {
            return Err(BcError::Error(
                "matrix multiplication requires non-empty matrices".to_string(),
            ));
        }
        if rhs.len() != lhs_cols {
            return Err(BcError::Error(
                "matrix multiplication requires left columns to equal right rows".to_string(),
            ));
        }
        let rhs_cols = rhs[0].len();
        let zero = ComplexNumber::from_real(Decimal::ZERO);
        let mut result = vec![vec![zero.clone(); rhs_cols]; lhs.len()];
        for (result_row, lhs_row) in result.iter_mut().zip(lhs.iter()) {
            for (j, result_cell) in result_row.iter_mut().enumerate() {
                let mut sum = ComplexNumber::from_real(Decimal::ZERO);
                for (lhs_value, rhs_row) in lhs_row.iter().zip(rhs.iter()) {
                    let product = lhs_value.mul(&rhs_row[j]);
                    sum = sum.add(&product);
                }
                *result_cell = sum;
            }
        }
        Ok(result)
    }

    pub(super) fn promote_matrix_precision(
        &self,
        matrix: Vec<Vec<ComplexNumber>>,
    ) -> Vec<Vec<ComplexNumber>> {
        matrix
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|value| self.promote_complex(value))
                    .collect()
            })
            .collect()
    }

    pub(super) fn promote_complex(&self, value: ComplexNumber) -> ComplexNumber {
        ComplexNumber {
            real: self.promote_precision(value.real),
            imag: self.promote_precision(value.imag),
        }
    }
}
