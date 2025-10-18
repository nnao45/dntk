use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::f64;
use std::fmt;

use fasteval::Evaler;
use fasteval::{Parser, Slab};
use libm::jn;
use rand::{rngs::SmallRng, Rng, RngCore, SeedableRng};
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
            BcError::Error(msg) => write!(f, "Evaluation error: {msg}"),
        }
    }
}

#[derive(Clone, Debug)]
struct FunctionDef {
    params: Vec<String>,
    body: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
enum StatementOutcome {
    None,
    Value(Decimal),
    Return(Decimal),
}

impl StatementOutcome {
    fn value(value: Decimal) -> Self {
        StatementOutcome::Value(value)
    }

    fn ret(value: Decimal) -> Self {
        StatementOutcome::Return(value)
    }

}

pub struct BcExecuter {
    parser: Parser,
    namespaces: Vec<BTreeMap<String, f64>>,
    functions: HashMap<String, FunctionDef>,
    scale: u32,
    obase: u32,
    rng: SmallRng,
}

impl fmt::Debug for BcExecuter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BcExecuter").finish()
    }
}

impl Default for BcExecuter {
    fn default() -> Self {
        let mut namespaces = vec![BTreeMap::new()];
        let scale = util::DNTK_OPT.scale as u32;
        namespaces[0].insert("scale".to_string(), scale as f64);
        namespaces[0].insert("obase".to_string(), 10.0);
        BcExecuter {
            parser: Parser::new(),
            namespaces,
            functions: HashMap::new(),
            scale,
            obase: 10,
            rng: SmallRng::seed_from_u64(0x5eed_5eed_5eed_5eed),
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
            match self.eval_statement(&stmt)? {
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

        let value = last_value.unwrap_or_else(Decimal::zero);
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
                    let outcome = self.eval_statement(&extra)?;
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
            let value = self.eval_assignment(&name, &expr)?;
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

        let mut else_branch: Vec<String> = Vec::new();
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
            return self.eval_block(then_branch);
        }
        if !else_branch.is_empty() {
            return self.eval_block(else_branch);
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

            match self.eval_block(body.clone())? {
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

            match self.eval_block(body.clone())? {
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

    fn eval_block(&mut self, statements: Vec<String>) -> Result<StatementOutcome, BcError> {
        let mut last_value = StatementOutcome::None;
        for stmt in statements {
            match self.eval_statement(&stmt)? {
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
            return Ok(StatementOutcome::ret(Decimal::zero()));
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
        let params = if params_str.trim().is_empty() {
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
        let body = self.split_statements(body_content);

        self.functions
            .insert(name.to_string(), FunctionDef { params, body });
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

        let value_f64 = value
            .to_f64()
            .ok_or_else(|| BcError::Error("Failed to convert to f64".to_string()))?;

        if let Some(scope) = self.find_scope_mut(name) {
            scope.insert(name.to_string(), value_f64);
        } else if let Some(scope) = self.namespaces.last_mut() {
            scope.insert(name.to_string(), value_f64);
        }

        Ok(value)
    }

    fn apply_special_assignment(&mut self, name: &str, value: &Decimal) -> Result<bool, BcError> {
        match name {
            "scale" => {
                if value.is_sign_negative() {
                    return Err(BcError::Error("scale() must be non-negative".to_string()));
                }
                let new_scale = value
                    .trunc()
                    .to_u32()
                    .ok_or_else(|| BcError::Error("scale() out of range".to_string()))?
                    .min(28);
                self.scale = new_scale;
                if let Some(scope) = self.namespaces.last_mut() {
                    scope.insert("scale".to_string(), new_scale as f64);
                }
                Ok(true)
            }
            "obase" => {
                if value.is_sign_negative() {
                    return Err(BcError::Error("obase must be positive".to_string()));
                }
                let new_obase = value
                    .trunc()
                    .to_u32()
                    .ok_or_else(|| BcError::Error("obase out of range".to_string()))?;
                if !(2..=36).contains(&new_obase) {
                    return Err(BcError::Error("obase must be between 2 and 36".to_string()));
                }
                self.obase = new_obase;
                if let Some(scope) = self.namespaces.last_mut() {
                    scope.insert("obase".to_string(), new_obase as f64);
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn find_scope_mut(&mut self, name: &str) -> Option<&mut BTreeMap<String, f64>> {
        self.namespaces
            .iter_mut()
            .rev()
            .find(|scope| scope.contains_key(name))
    }

    fn eval_expression(&mut self, expr: &str) -> Result<Decimal, BcError> {
        let processed = self.preprocess_bc_syntax(expr);
        let mut slab = Slab::new();
        let expr_idx = self
            .parser
            .parse(&processed, &mut slab.ps)
            .map_err(|e| BcError::Error(e.to_string()))?;
        let expr_ref = expr_idx.from(&slab.ps);

        let error = RefCell::new(None::<BcError>);
        let self_ptr: *mut BcExecuter = self;
        let mut namespace = |name: &str, args: Vec<f64>| -> Option<f64> {
            if error.borrow().is_some() {
                return None;
            }
            // Safety: namespace closure only runs while we hold exclusive &mut self.
            match unsafe { (*self_ptr).resolve_name(name, args) } {
                Ok(val) => Some(val),
                Err(err) => {
                    *error.borrow_mut() = Some(err);
                    None
                }
            }
        };

        let result = expr_ref
            .eval(&slab, &mut namespace)
            .map_err(|e| BcError::Error(e.to_string()))?;

        if let Some(err) = error.into_inner() {
            return Err(err);
        }

        Decimal::from_f64(result)
            .ok_or_else(|| BcError::Error("Failed to convert to decimal".to_string()))
    }

    fn resolve_name(&mut self, name: &str, args: Vec<f64>) -> Result<f64, BcError> {
        if args.is_empty() {
            if let Some(value) = self.lookup_variable(name) {
                return Ok(value);
            }
        }

        if let Some(result) = self.call_builtin_function(name, &args) {
            return result;
        }

        if let Some(func_value) = self.call_function(name, args)? {
            return func_value
                .to_f64()
                .ok_or_else(|| BcError::Error("Failed to convert function result".to_string()));
        }

        Err(BcError::Error(format!("Undefined identifier: {name}")))
    }

    fn lookup_variable(&self, name: &str) -> Option<f64> {
        for scope in self.namespaces.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(*val);
            }
        }
        None
    }

    fn call_function(&mut self, name: &str, args: Vec<f64>) -> Result<Option<Decimal>, BcError> {
        let def = match self.functions.get(name) {
            Some(def) => def.clone(),
            None => return Ok(None),
        };

        if args.len() != def.params.len() {
            return Err(BcError::Error(format!(
                "Function {} expected {} arguments, got {}",
                name,
                def.params.len(),
                args.len()
            )));
        }

        let mut local_scope = BTreeMap::new();
        for (param, arg) in def.params.iter().zip(args.iter()) {
            local_scope.insert(param.clone(), *arg);
        }

        self.namespaces.push(local_scope);
        let outcome = self.eval_block(def.body)?;
        self.namespaces.pop();

        let result = match outcome {
            StatementOutcome::Return(value) | StatementOutcome::Value(value) => value,
            StatementOutcome::None => Decimal::zero(),
        };

        Ok(Some(result))
    }

    fn call_builtin_function(
        &mut self,
        name: &str,
        args: &[f64],
    ) -> Option<Result<f64, BcError>> {
        match name {
            "length" => Some(Self::builtin_length(args)),
            "scale" => Some(Self::builtin_scale(args)),
            "j" => Some(Self::builtin_bessel(args)),
            "rand" => Some(self.builtin_rand(args)),
            "srand" => Some(self.builtin_srand(args)),
            "sqrt" => Some(Self::builtin_unary("sqrt", args, libm::sqrt)),
            "cbrt" => Some(Self::builtin_unary("cbrt", args, libm::cbrt)),
            "abs" => Some(Self::builtin_unary("abs", args, libm::fabs)),
            "sign" => Some(Self::builtin_sign(args)),
            "floor" => Some(Self::builtin_unary("floor", args, libm::floor)),
            "ceil" => Some(Self::builtin_unary("ceil", args, libm::ceil)),
            "trunc" => Some(Self::builtin_unary("trunc", args, libm::trunc)),
            "round" => Some(Self::builtin_unary("round", args, libm::round)),
            "sin" => Some(Self::builtin_unary("sin", args, libm::sin)),
            "cos" => Some(Self::builtin_unary("cos", args, libm::cos)),
            "tan" => Some(Self::builtin_unary("tan", args, libm::tan)),
            "asin" | "arcsin" => Some(Self::builtin_unary("asin", args, libm::asin)),
            "acos" | "arccos" => Some(Self::builtin_unary("acos", args, libm::acos)),
            "atan" | "arctan" => Some(Self::builtin_unary("atan", args, libm::atan)),
            "atan2" => Some(Self::builtin_binary("atan2", args, libm::atan2)),
            "sinh" => Some(Self::builtin_unary("sinh", args, libm::sinh)),
            "cosh" => Some(Self::builtin_unary("cosh", args, libm::cosh)),
            "tanh" => Some(Self::builtin_unary("tanh", args, libm::tanh)),
            "asinh" => Some(Self::builtin_unary("asinh", args, libm::asinh)),
            "acosh" => Some(Self::builtin_unary("acosh", args, libm::acosh)),
            "atanh" => Some(Self::builtin_unary("atanh", args, libm::atanh)),
            "exp" => Some(Self::builtin_unary("exp", args, libm::exp)),
            "expm1" => Some(Self::builtin_unary("expm1", args, libm::expm1)),
            "ln" => Some(Self::builtin_unary("ln", args, libm::log)),
            "log" => Some(Self::builtin_log(args)),
            "log10" => Some(Self::builtin_unary("log10", args, libm::log10)),
            "log2" => Some(Self::builtin_unary("log2", args, libm::log2)),
            "pow" => Some(Self::builtin_binary("pow", args, libm::pow)),
            "hypot" => Some(Self::builtin_binary("hypot", args, libm::hypot)),
            "min" => Some(Self::builtin_min(args)),
            "max" => Some(Self::builtin_max(args)),
            _ => None,
        }
    }

    fn builtin_length(args: &[f64]) -> Result<f64, BcError> {
        if args.len() != 1 {
            return Err(BcError::Error(format!(
                "length() expects 1 argument, got {}",
                args.len()
            )));
        }
        let decimal = Decimal::from_f64(args[0])
            .ok_or_else(|| BcError::Error("Failed to convert argument to decimal".to_string()))?;
        let mut digits = decimal
            .normalize()
            .abs()
            .to_string()
            .chars()
            .filter(|c| *c != '-' && *c != '.')
            .collect::<String>();
        if digits.is_empty() {
            digits.push('0');
        }
        Ok(digits.chars().count() as f64)
    }

    fn builtin_scale(args: &[f64]) -> Result<f64, BcError> {
        if args.len() != 1 {
            return Err(BcError::Error(format!(
                "scale() expects 1 argument, got {}",
                args.len()
            )));
        }
        let decimal = Decimal::from_f64(args[0])
            .ok_or_else(|| BcError::Error("Failed to convert argument to decimal".to_string()))?;
        Ok(decimal.normalize().scale() as f64)
    }

    fn builtin_bessel(args: &[f64]) -> Result<f64, BcError> {
        if args.len() != 2 {
            return Err(BcError::Error(format!(
                "j() expects 2 arguments, got {}",
                args.len()
            )));
        }
        let order_float = args[0];
        let order_rounded = order_float.round();
        if (order_float - order_rounded).abs() > f64::EPSILON {
            return Err(BcError::Error(
                "Bessel function order must be an integer".to_string(),
            ));
        }
        let order = order_rounded as i32;
        let value = jn(order, args[1]);
        if !value.is_finite() {
            return Err(BcError::Error(
                "Bessel function produced a non-finite result".to_string(),
            ));
        }
        Ok(value)
    }

    fn builtin_rand(&mut self, args: &[f64]) -> Result<f64, BcError> {
        match args.len() {
            0 => Ok((self.rng.next_u32() & 0x7fff) as f64),
            1 => {
                let limit = args[0];
                if limit <= 0.0 {
                    return Err(BcError::Error("rand(n) expects n > 0".to_string()));
                }
                let upper = limit.floor() as u32;
                if upper == 0 {
                    return Ok(0.0);
                }
                Ok(self.rng.gen_range(0..upper) as f64)
            }
              n => Err(BcError::Error(format!(
                "rand() expects 0 or 1 arguments, got {n}",
            ))),
        }
    }

    fn builtin_srand(&mut self, args: &[f64]) -> Result<f64, BcError> {
        if args.len() != 1 {
            return Err(BcError::Error("srand(seed) expects exactly 1 argument".to_string()));
        }
        let seed_val = args[0].trunc();
        if seed_val.is_sign_negative() {
            return Err(BcError::Error("srand(seed) expects non-negative seed".to_string()));
        }
        let seed = seed_val as u64;
        self.rng = SmallRng::seed_from_u64(seed);
        Ok(seed as f64)
    }

    fn builtin_unary<F>(name: &str, args: &[f64], func: F) -> Result<f64, BcError>
    where
        F: Fn(f64) -> f64,
    {
        if args.len() != 1 {
            return Err(BcError::Error(format!(
                "{}() expects 1 argument, got {}",
                name,
                args.len()
            )));
        }
        Self::validate_finite(name, func(args[0]))
    }

    fn builtin_binary<F>(name: &str, args: &[f64], func: F) -> Result<f64, BcError>
    where
        F: Fn(f64, f64) -> f64,
    {
        if args.len() != 2 {
            return Err(BcError::Error(format!(
                "{}() expects 2 arguments, got {}",
                name,
                args.len()
            )));
        }
        Self::validate_finite(name, func(args[0], args[1]))
    }

    fn builtin_log(args: &[f64]) -> Result<f64, BcError> {
        match args.len() {
            1 => {
                let value = args[0];
                if value <= 0.0 {
                    return Err(BcError::Error("log() expects positive input".to_string()));
                }
                Self::validate_finite("log", libm::log10(value))
            }
            2 => {
                let base = args[0];
                let value = args[1];
                if value <= 0.0 {
                    return Err(BcError::Error("log() expects positive argument".to_string()));
                }
                if base <= 0.0 || (base - 1.0).abs() < f64::EPSILON {
                    return Err(BcError::Error(
                        "log() base must be positive and not equal to 1".to_string(),
                    ));
                }
                let result = libm::log(value) / libm::log(base);
                Self::validate_finite("log", result)
            }
            n => Err(BcError::Error(format!(
                "log() expects 1 or 2 arguments, got {n}",
            ))),
        }
    }

    fn builtin_sign(args: &[f64]) -> Result<f64, BcError> {
        if args.len() != 1 {
            return Err(BcError::Error(format!(
                "sign() expects 1 argument, got {}",
                args.len()
            )));
        }
        let x = args[0];
        Ok(if x > 0.0 {
            1.0
        } else if x < 0.0 {
            -1.0
        } else {
            0.0
        })
    }

    fn builtin_min(args: &[f64]) -> Result<f64, BcError> {
        if args.len() < 2 {
            return Err(BcError::Error("min() expects at least 2 arguments".to_string()));
        }
        let mut value = args[0];
        for &arg in &args[1..] {
            value = libm::fmin(value, arg);
        }
        Self::validate_finite("min", value)
    }

    fn builtin_max(args: &[f64]) -> Result<f64, BcError> {
        if args.len() < 2 {
            return Err(BcError::Error("max() expects at least 2 arguments".to_string()));
        }
        let mut value = args[0];
        for &arg in &args[1..] {
            value = libm::fmax(value, arg);
        }
        Self::validate_finite("max", value)
    }

    fn validate_finite(name: &str, value: f64) -> Result<f64, BcError> {
        if value.is_finite() {
            Ok(value)
        } else {
            Err(BcError::Error(format!(
                "{name}() produced a non-finite result",
            )))
        }
    }

    fn parse_branch<'a>(&self, input: &'a str) -> Result<(Vec<String>, &'a str), BcError> {
        let trimmed = input.trim_start();
        if trimmed.starts_with('{') {
            let end = Self::find_matching(trimmed, 0, '{', '}')?;
            let body = &trimmed[1..end];
            let remainder = &trimmed[end + 1..];
            let statements = self.split_statements(body);
            Ok((statements, remainder))
        } else {
            let chars: Vec<char> = trimmed.chars().collect();
            let mut idx = 0usize;
            let mut depth_round = 0;
            let mut depth_square = 0;
            let mut depth_curly = 0;

            while idx < chars.len() {
                match chars[idx] {
                    '(' => depth_round += 1,
                    ')' => if depth_round > 0 { depth_round -= 1; },
                    '[' => depth_square += 1,
                    ']' => if depth_square > 0 { depth_square -= 1; },
                    '{' => depth_curly += 1,
                    '}' => if depth_curly > 0 { depth_curly -= 1; },
                    'e' | 'E' if depth_round == 0 && depth_square == 0 && depth_curly == 0 => {
                        if trimmed[idx..].starts_with("else")
                            && Self::is_keyword_boundary(trimmed, idx, idx + 4)
                        {
                            break;
                        }
                    }
                    ';' if depth_round == 0 && depth_square == 0 && depth_curly == 0 => {
                        idx += 1;
                        break;
                    }
                    _ => {}
                }
                idx += 1;
            }

            let statement = trimmed[..idx].trim();
            let remainder = trimmed[idx..].trim_start();
            let statements = if statement.is_empty() {
                Vec::new()
            } else {
                vec![statement.to_string()]
            };
            Ok((statements, remainder))
        }
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
                ')' => if depth_round > 0 { depth_round -= 1; },
                '[' => depth_square += 1,
                ']' => if depth_square > 0 { depth_square -= 1; },
                '{' => depth_curly += 1,
                '}' => if depth_curly > 0 { depth_curly -= 1; },
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

    fn detect_assignment(stmt: &str) -> Option<(String, String)> {
        let mut depth_round = 0;
        let mut depth_square = 0;
        let mut depth_curly = 0;
        let chars: Vec<char> = stmt.chars().collect();

        for (index, ch) in chars.iter().enumerate() {
            match ch {
                '(' => depth_round += 1,
                ')' => if depth_round > 0 { depth_round -= 1; },
                '[' => depth_square += 1,
                ']' => if depth_square > 0 { depth_square -= 1; },
                '{' => depth_curly += 1,
                '}' => if depth_curly > 0 { depth_curly -= 1; },
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

    fn split_top_level(input: &str, delimiter: char) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut depth_round = 0;
        let mut depth_square = 0;
        let mut depth_curly = 0;

        for ch in input.chars() {
            match ch {
                '(' => depth_round += 1,
                ')' => if depth_round > 0 { depth_round -= 1; },
                '[' => depth_square += 1,
                ']' => if depth_square > 0 { depth_square -= 1; },
                '{' => depth_curly += 1,
                '}' => if depth_curly > 0 { depth_curly -= 1; },
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

    fn lookup_keyword(bytes: &str, expected: &str) -> bool {
        bytes.len() >= expected.len()
            && bytes[..expected.len()].eq_ignore_ascii_case(expected)
            && Self::is_keyword_boundary(bytes, 0, expected.len())
    }

    fn starts_with_keyword(input: &str, keyword: &str) -> bool {
        let trimmed = input.trim_start();
        Self::lookup_keyword(trimmed, keyword)
    }

    fn is_keyword_boundary(input: &str, start: usize, end: usize) -> bool {
        let bytes = input.as_bytes();
        let before = start.checked_sub(1).and_then(|idx| bytes.get(idx));
        let after = bytes.get(end);

        let prev_ok = before.is_none_or(|c| !Self::is_ident_char(*c));
        let next_ok = after.is_none_or(|c| !Self::is_ident_char(*c));
        prev_ok && next_ok
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

    fn format_result(&self, value: Decimal) -> String {
        if self.obase == 10 {
            return self.format_result_decimal(value);
        }
        if let Some(formatted) = self.format_result_obase(value) {
            formatted
        } else {
            self.format_result_decimal(value)
        }
    }

    fn format_result_decimal(&self, value: Decimal) -> String {
        let scale = self.scale;
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

    fn format_result_obase(&self, value: Decimal) -> Option<String> {
        const DIGITS: &[u8; 36] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let base = self.obase as i32;
        let negative = value.is_sign_negative();
        let abs_value = value.abs();
        let integer_part = abs_value.trunc();
        let mut integer = integer_part.to_i128()?;

        let mut int_buf = Vec::new();
        if integer == 0 {
            int_buf.push('0');
        } else {
            while integer > 0 {
                let digit = (integer % base as i128) as usize;
                int_buf.push(DIGITS[digit] as char);
                integer /= base as i128;
            }
            int_buf.reverse();
        }

        let mut result = String::new();
        if negative && (!int_buf.is_empty() || !abs_value.fract().is_zero()) {
            result.push('-');
        }
        for ch in int_buf {
            result.push(ch);
        }

        let mut fraction = abs_value - integer_part;
        if !fraction.is_zero() && self.scale > 0 {
            result.push('.');
            let base_decimal = Decimal::from(base as i64);
            let mut digits_written = 0;
            while digits_written < self.scale {
                fraction *= base_decimal;
                let digit_dec = fraction.trunc();
                let digit = digit_dec.to_u32()? as usize;
                result.push(DIGITS[digit] as char);
                fraction -= digit_dec;
                digits_written += 1;
                if fraction.is_zero() {
                    break;
                }
            }
            while result.ends_with('0') {
                result.pop();
            }
            if result.ends_with('.') {
                result.pop();
            }
        }

        if result.is_empty() || result == "-" {
            result.push('0');
        }

        Some(result)
    }

    fn preprocess_bc_syntax(&self, statement: &str) -> String {
        let bytes = statement.as_bytes();
        let mut result = String::with_capacity(statement.len());
        let mut i = 0;
        while i < bytes.len() {
            if i + 1 < bytes.len()
                && bytes[i + 1] == b'('
                && !Self::has_ident_before(bytes, i)
            {
                let replacement = match bytes[i] {
                    b's' => Some("sin("),
                    b'c' => Some("cos("),
                    b'a' => Some("atan("),
                    b'l' => Some("ln("),
                    b'e' => Some("exp("),
                    _ => None,
                };
                if let Some(rep) = replacement {
                    result.push_str(rep);
                    i += 2;
                    continue;
                }
            }
            result.push(bytes[i] as char);
            i += 1;
        }
        result
    }

    fn is_valid_identifier(name: &str) -> bool {
        let mut chars = name.chars();
        match chars.next() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => (),
            _ => return false,
        }
        chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    fn is_ident_char(byte: u8) -> bool {
        byte.is_ascii_alphanumeric() || byte == b'_'
    }

    fn has_ident_before(bytes: &[u8], idx: usize) -> bool {
        if idx == 0 {
            return false;
        }
        Self::is_ident_char(bytes[idx - 1])
    }
}
