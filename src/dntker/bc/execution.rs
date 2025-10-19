use std::fmt;

use dashu::base::Sign;
use dashu::Decimal;
use fasteval::Parser;
use num_traits::{ToPrimitive, Zero};

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
}
