use std::collections::{BTreeMap, HashMap};
use std::f64;
use std::fmt;

use dashu::base::{Abs, Approximation, Sign, UnsignedAbs};
use dashu::Decimal;
use fasteval::compiler::{Compiler, Instruction, InstructionI, IC};
use fasteval::slab::CompileSlab;
use fasteval::{Parser, Slab};
use libm::jn;
use num_traits::{ToPrimitive, Zero};
use rand::{rngs::SmallRng, Rng, RngCore, SeedableRng};

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

#[derive(Clone, Debug)]
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
    namespaces: Vec<BTreeMap<String, Decimal>>,
    functions: HashMap<String, FunctionDef>,
    scale: u32,
    obase: u32,
    rng: SmallRng,
    literal_values: HashMap<String, Decimal>,
    literal_counter: usize,
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
        namespaces[0].insert("scale".to_string(), Decimal::from(scale));
        namespaces[0].insert("obase".to_string(), Decimal::from(10));
        BcExecuter {
            parser: Parser::new(),
            namespaces,
            functions: HashMap::new(),
            scale,
            obase: 10,
            rng: SmallRng::seed_from_u64(0x5eed_5eed_5eed_5eed),
            literal_values: HashMap::new(),
            literal_counter: 0,
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

        let value = last_value.unwrap_or_else(|| Decimal::ZERO);
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

        if let Some(scope) = self.find_scope_mut(name) {
            scope.insert(name.to_string(), value.clone());
        } else if let Some(scope) = self.namespaces.last_mut() {
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
                self.scale = new_scale;
                if let Some(scope) = self.namespaces.last_mut() {
                    scope.insert("scale".to_string(), Decimal::from(new_scale));
                }
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
                self.obase = new_obase;
                if let Some(scope) = self.namespaces.last_mut() {
                    scope.insert("obase".to_string(), Decimal::from(new_obase));
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn find_scope_mut(&mut self, name: &str) -> Option<&mut BTreeMap<String, Decimal>> {
        self.namespaces
            .iter_mut()
            .rev()
            .find(|scope| scope.contains_key(name))
    }

    fn eval_expression(&mut self, expr: &str) -> Result<Decimal, BcError> {
        let trimmed = expr.trim();

        // Try to parse as a simple numeric literal first (preserves full precision)
        if let Ok(decimal_value) = trimmed.parse::<Decimal>() {
            return Ok(self.promote_precision(decimal_value));
        }

        self.literal_values.clear();
        self.literal_counter = 0;
        let processed = self.preprocess_bc_syntax(expr);
        let substituted = self.substitute_numeric_literals(&processed)?;
        let mut slab = Slab::new();
        let expr_idx = self
            .parser
            .parse(&substituted, &mut slab.ps)
            .map_err(|e| BcError::Error(e.to_string()))?;
        let expression = expr_idx.from(&slab.ps);
        let instruction = expression.compile(&slab.ps, &mut slab.cs);
        let value = self.eval_instruction(&instruction, &slab.cs)?;
        self.literal_values.clear();
        Ok(self.promote_precision(value))
    }



    fn eval_instruction(
        &mut self,
        instruction: &Instruction,
        compile_slab: &CompileSlab,
    ) -> Result<Decimal, BcError> {
        let value = match instruction {
            Instruction::IConst(value) => self.decimal_from_f64(*value, "Failed to convert constant"),
            Instruction::INeg(idx) => Ok(-self.eval_instruction_index(*idx, compile_slab)?),
            Instruction::INot(idx) => {
                let value = self.eval_instruction_index(*idx, compile_slab)?;
                Ok(Self::bool_to_decimal(value.is_zero()))
            }
            Instruction::IInv(idx) => {
                let value = self.eval_instruction_index(*idx, compile_slab)?;
                if value.is_zero() {
                    return Err(BcError::Error("Division by zero".to_string()));
                }
                Ok(Decimal::from(1) / value)
            }
            Instruction::IAdd(lhs, rhs) => {
                let left = self.eval_instruction_index(*lhs, compile_slab)?;
                let right = self.eval_ic(rhs, compile_slab)?;
                Ok(left + right)
            }
            Instruction::IMul(lhs, rhs) => {
                let left = self.eval_instruction_index(*lhs, compile_slab)?;
                let right = self.eval_ic(rhs, compile_slab)?;
                Ok(left * right)
            }
            Instruction::IMod { dividend, divisor } => {
                let left = self.eval_ic(dividend, compile_slab)?;
                let right = self.eval_ic(divisor, compile_slab)?;
                if right.is_zero() {
                    return Err(BcError::Error("Modulo by zero".to_string()));
                }
                Ok(left % right)
            }
            Instruction::IExp { base, power } => {
                let left = self.eval_ic(base, compile_slab)?;
                let right = self.eval_ic(power, compile_slab)?;
                self.power_decimal(&left, &right)
            }
            Instruction::ILT(lhs, rhs) => {
                let left = self.eval_ic(lhs, compile_slab)?;
                let right = self.eval_ic(rhs, compile_slab)?;
                Ok(Self::bool_to_decimal(left < right))
            }
            Instruction::ILTE(lhs, rhs) => {
                let left = self.eval_ic(lhs, compile_slab)?;
                let right = self.eval_ic(rhs, compile_slab)?;
                Ok(Self::bool_to_decimal(left <= right))
            }
            Instruction::IEQ(lhs, rhs) => {
                let left = self.eval_ic(lhs, compile_slab)?;
                let right = self.eval_ic(rhs, compile_slab)?;
                Ok(Self::bool_to_decimal(left == right))
            }
            Instruction::INE(lhs, rhs) => {
                let left = self.eval_ic(lhs, compile_slab)?;
                let right = self.eval_ic(rhs, compile_slab)?;
                Ok(Self::bool_to_decimal(left != right))
            }
            Instruction::IGTE(lhs, rhs) => {
                let left = self.eval_ic(lhs, compile_slab)?;
                let right = self.eval_ic(rhs, compile_slab)?;
                Ok(Self::bool_to_decimal(left >= right))
            }
            Instruction::IGT(lhs, rhs) => {
                let left = self.eval_ic(lhs, compile_slab)?;
                let right = self.eval_ic(rhs, compile_slab)?;
                Ok(Self::bool_to_decimal(left > right))
            }
            Instruction::IOR(lhs, rhs) => {
                let left = self.eval_instruction_index(*lhs, compile_slab)?;
                if Self::decimal_truth(&left) {
                    return Ok(Decimal::from(1));
                }
                let right = self.eval_ic(rhs, compile_slab)?;
                Ok(Self::bool_to_decimal(Self::decimal_truth(&right)))
            }
            Instruction::IAND(lhs, rhs) => {
                let left = self.eval_instruction_index(*lhs, compile_slab)?;
                if !Self::decimal_truth(&left) {
                    return Ok(Decimal::ZERO);
                }
                let right = self.eval_ic(rhs, compile_slab)?;
                Ok(Self::bool_to_decimal(Self::decimal_truth(&right)))
            }
            Instruction::IVar(name) => self.resolve_name(name, Vec::new()),
            Instruction::IFunc { name, args } => {
                let mut evaluated = Vec::with_capacity(args.len());
                for arg in args {
                    evaluated.push(self.eval_ic(arg, compile_slab)?);
                }
                self.resolve_name(name, evaluated)
            }
            Instruction::IFuncInt(idx) => {
                let value = self.eval_instruction_index(*idx, compile_slab)?;
                Ok(value.trunc())
            }
            Instruction::IFuncCeil(idx) => {
                let value = self.eval_instruction_index(*idx, compile_slab)?;
                Ok(value.ceil())
            }
            Instruction::IFuncFloor(idx) => {
                let value = self.eval_instruction_index(*idx, compile_slab)?;
                Ok(value.floor())
            }
            Instruction::IFuncAbs(idx) => {
                let value = self.eval_instruction_index(*idx, compile_slab)?;
                Ok(value.abs())
            }
            Instruction::IFuncSign(idx) => {
                let value = self.eval_instruction_index(*idx, compile_slab)?;
                Ok(Self::decimal_sign(&value))
            }
            Instruction::IFuncLog { base, of } => {
                let base_val = self.eval_ic(base, compile_slab)?;
                let value = self.eval_ic(of, compile_slab)?;
                Self::builtin_log_decimal(&base_val, &value)
            }
            Instruction::IFuncRound { modulus, of } => {
                let modulus_val = self.eval_ic(modulus, compile_slab)?;
                if modulus_val.is_zero() {
                    return Err(BcError::Error("round() expects non-zero modulus".to_string()));
                }
                let value = self.eval_ic(of, compile_slab)?;
                let quotient = value.clone() / modulus_val.clone();
                Ok(quotient.round() * modulus_val)
            }
            Instruction::IFuncMin(first, rest) => {
                let mut current = self.eval_instruction_index(*first, compile_slab)?;
                let candidate = self.eval_ic(rest, compile_slab)?;
                if candidate < current {
                    current = candidate;
                }
                Ok(current)
            }
            Instruction::IFuncMax(first, rest) => {
                let mut current = self.eval_instruction_index(*first, compile_slab)?;
                let candidate = self.eval_ic(rest, compile_slab)?;
                if candidate > current {
                    current = candidate;
                }
                Ok(current)
            }
            Instruction::IFuncSin(idx) => self.eval_math_function(*idx, compile_slab, libm::sin, "sin"),
            Instruction::IFuncCos(idx) => self.eval_math_function(*idx, compile_slab, libm::cos, "cos"),
            Instruction::IFuncTan(idx) => self.eval_math_function(*idx, compile_slab, libm::tan, "tan"),
            Instruction::IFuncASin(idx) => self.eval_math_function(*idx, compile_slab, libm::asin, "asin"),
            Instruction::IFuncACos(idx) => self.eval_math_function(*idx, compile_slab, libm::acos, "acos"),
            Instruction::IFuncATan(idx) => self.eval_math_function(*idx, compile_slab, libm::atan, "atan"),
            Instruction::IFuncSinH(idx) => self.eval_math_function(*idx, compile_slab, libm::sinh, "sinh"),
            Instruction::IFuncCosH(idx) => self.eval_math_function(*idx, compile_slab, libm::cosh, "cosh"),
            Instruction::IFuncTanH(idx) => self.eval_math_function(*idx, compile_slab, libm::tanh, "tanh"),
            Instruction::IFuncASinH(idx) => self.eval_math_function(*idx, compile_slab, libm::asinh, "asinh"),
            Instruction::IFuncACosH(idx) => self.eval_math_function(*idx, compile_slab, libm::acosh, "acosh"),
            Instruction::IFuncATanH(idx) => self.eval_math_function(*idx, compile_slab, libm::atanh, "atanh"),
            Instruction::IPrintFunc(_) => Err(BcError::Error("print() is not supported".to_string())),
        }?;
        Ok(self.promote_precision(value))
    }

    fn eval_instruction_index(
        &mut self,
        idx: InstructionI,
        compile_slab: &CompileSlab,
    ) -> Result<Decimal, BcError> {
        let instruction = compile_slab.get_instr(idx);
        self.eval_instruction(instruction, compile_slab)
    }

    fn eval_ic(&mut self, ic: &IC, compile_slab: &CompileSlab) -> Result<Decimal, BcError> {
        match ic {
            IC::C(value) => self.decimal_from_f64(*value, "Failed to convert constant"),
            IC::I(index) => self.eval_instruction_index(*index, compile_slab),
        }
    }

    fn eval_math_function(
        &mut self,
        idx: InstructionI,
        compile_slab: &CompileSlab,
        func: impl Fn(f64) -> f64,
        name: &str,
    ) -> Result<Decimal, BcError> {
        let value = self.eval_instruction_index(idx, compile_slab)?;
        let input = Self::decimal_to_f64(&value, &format!("{name}() argument out of range"))?;
        let result = func(input);
        self.decimal_from_f64(result, &format!("{name}() produced invalid result"))
    }

    fn power_decimal(&self, base: &Decimal, exponent: &Decimal) -> Result<Decimal, BcError> {
        if exponent.fract().is_zero() {
            let power = ToPrimitive::to_i64(&exponent.trunc()).ok_or_else(|| {
                BcError::Error("Exponent out of supported range".to_string())
            })?;
            Ok(base.clone().powi(power.into()))
        } else {
            let base_f = Self::decimal_to_f64(base, "Exponentiation base out of range")?;
            let exponent_f = Self::decimal_to_f64(exponent, "Exponentiation power out of range")?;
            let result = libm::pow(base_f, exponent_f);
            self.decimal_from_f64(result, "Exponentiation produced invalid result")
        }
    }

    fn builtin_log_decimal(base: &Decimal, value: &Decimal) -> Result<Decimal, BcError> {
        let base_f = Self::decimal_to_f64(base, "log() base out of range")?;
        let value_f = Self::decimal_to_f64(value, "log() expects positive argument")?;
        if value_f <= 0.0 {
            return Err(BcError::Error("log() expects positive argument".to_string()));
        }
        if base_f <= 0.0 || (base_f - 1.0).abs() < f64::EPSILON {
            return Err(BcError::Error(
                "log() base must be positive and not equal to 1".to_string(),
            ));
        }
        let result = libm::log(value_f) / libm::log(base_f);
        Self::decimal_from_f64_static(result, "log() produced invalid result")
    }

    fn bool_to_decimal(value: bool) -> Decimal {
        if value {
            Decimal::from(1)
        } else {
            Decimal::ZERO
        }
    }

    fn decimal_truth(value: &Decimal) -> bool {
        !value.is_zero()
    }

    fn decimal_sign(value: &Decimal) -> Decimal {
        if value.is_zero() {
            Decimal::ZERO
        } else {
            match value.sign() {
                Sign::Positive => Decimal::from(1),
                Sign::Negative => Decimal::from(-1),
            }
        }
    }

    fn decimal_to_f64(value: &Decimal, err: &str) -> Result<f64, BcError> {
        ToPrimitive::to_f64(value).ok_or_else(|| BcError::Error(err.to_string()))
    }
    fn resolve_name(&mut self, name: &str, args: Vec<Decimal>) -> Result<Decimal, BcError> {
        if args.is_empty() {
            if let Some(value) = self.literal_values.get(name) {
                return Ok(value.clone());
            }
            if let Some(value) = self.lookup_variable(name) {
                return Ok(value);
            }
        }

        if let Some(result) = self.call_builtin_function(name, &args) {
            return result;
        }

        if let Some(func_value) = self.call_function(name, args)? {
            return Ok(func_value);
        }

        Err(BcError::Error(format!("Undefined identifier: {name}")))
    }

    fn lookup_variable(&self, name: &str) -> Option<Decimal> {
        for scope in self.namespaces.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val.clone());
            }
        }
        None
    }

    fn call_function(&mut self, name: &str, args: Vec<Decimal>) -> Result<Option<Decimal>, BcError> {
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
            local_scope.insert(param.clone(), arg.clone());
        }

        self.namespaces.push(local_scope);
        let outcome = self.eval_block(def.body)?;
        self.namespaces.pop();

        let result = match outcome {
            StatementOutcome::Return(value) | StatementOutcome::Value(value) => value,
            StatementOutcome::None => Decimal::ZERO,
        };

        Ok(Some(result))
    }

    fn call_builtin_function(
        &mut self,
        name: &str,
        args: &[Decimal],
    ) -> Option<Result<Decimal, BcError>> {
        let result = match name {
            "length" => Some(Self::builtin_length(args)),
            "scale" => Some(Self::builtin_scale(args)),
            "j" => Some(Self::builtin_bessel(args)),
            "rand" => Some(self.builtin_rand(args)),
            "srand" => Some(self.builtin_srand(args)),
            "sqrt" => Some(Self::builtin_math_unary("sqrt", args, libm::sqrt)),
            "cbrt" => Some(Self::builtin_math_unary("cbrt", args, libm::cbrt)),
            "abs" => Some(Self::builtin_abs(args)),
            "sign" => Some(Self::builtin_sign(args)),
            "floor" => Some(Self::builtin_decimal_unary("floor", args, |v| v.floor())),
            "ceil" => Some(Self::builtin_decimal_unary("ceil", args, |v| v.ceil())),
            "trunc" => Some(Self::builtin_decimal_unary("trunc", args, |v| v.trunc())),
            "round" => Some(Self::builtin_decimal_unary("round", args, |v| v.round())),
            "sin" => Some(Self::builtin_math_unary("sin", args, libm::sin)),
            "cos" => Some(Self::builtin_math_unary("cos", args, libm::cos)),
            "tan" => Some(Self::builtin_math_unary("tan", args, libm::tan)),
            "asin" | "arcsin" => Some(Self::builtin_math_unary("asin", args, libm::asin)),
            "acos" | "arccos" => Some(Self::builtin_math_unary("acos", args, libm::acos)),
            "atan" | "arctan" => Some(Self::builtin_math_unary("atan", args, libm::atan)),
            "atan2" => Some(Self::builtin_math_binary("atan2", args, libm::atan2)),
            "sinh" => Some(Self::builtin_math_unary("sinh", args, libm::sinh)),
            "cosh" => Some(Self::builtin_math_unary("cosh", args, libm::cosh)),
            "tanh" => Some(Self::builtin_math_unary("tanh", args, libm::tanh)),
            "asinh" => Some(Self::builtin_math_unary("asinh", args, libm::asinh)),
            "acosh" => Some(Self::builtin_math_unary("acosh", args, libm::acosh)),
            "atanh" => Some(Self::builtin_math_unary("atanh", args, libm::atanh)),
            "exp" => Some(Self::builtin_math_unary("exp", args, libm::exp)),
            "expm1" => Some(Self::builtin_math_unary("expm1", args, libm::expm1)),
            "ln" => Some(Self::builtin_math_unary("ln", args, libm::log)),
            "log" => Some(Self::builtin_log(args)),
            "log10" => Some(Self::builtin_math_unary("log10", args, libm::log10)),
            "log2" => Some(Self::builtin_math_unary("log2", args, libm::log2)),
            "pow" => Some(Self::builtin_pow(args)),
            "hypot" => Some(Self::builtin_math_binary("hypot", args, libm::hypot)),
            "min" => Some(Self::builtin_min(args)),
            "max" => Some(Self::builtin_max(args)),
            _ => None,
        };

        result.map(|res| res.map(|value| self.promote_precision(value)))
    }

    fn builtin_length(args: &[Decimal]) -> Result<Decimal, BcError> {
        if args.len() != 1 {
            return Err(BcError::Error(format!(
                "length() expects 1 argument, got {}",
                args.len()
            )));
        }
        let normalized = Self::decimal_to_plain_string(&args[0].clone().abs());
        let digits = normalized
            .chars()
            .filter(|c| c.is_ascii_digit())
            .count()
            .max(1);
        Ok(Decimal::from(digits as i64))
    }

    fn builtin_scale(args: &[Decimal]) -> Result<Decimal, BcError> {
        if args.len() != 1 {
            return Err(BcError::Error(format!(
                "scale() expects 1 argument, got {}",
                args.len()
            )));
        }
        let exponent = args[0].repr().exponent();
        if exponent >= 0 {
            Ok(Decimal::ZERO)
        } else {
            Ok(Decimal::from((-exponent) as i64))
        }
    }

    fn builtin_bessel(args: &[Decimal]) -> Result<Decimal, BcError> {
        if args.len() != 2 {
            return Err(BcError::Error(format!(
                "j() expects 2 arguments, got {}",
                args.len()
            )));
        }
        let order = Self::decimal_to_f64(&args[0], "Bessel order out of range")?;
        let rounded = order.round();
        if (order - rounded).abs() > f64::EPSILON {
            return Err(BcError::Error(
                "Bessel function order must be an integer".to_string(),
            ));
        }
        let order_int = rounded as i32;
        let argument = Self::decimal_to_f64(&args[1], "Bessel argument out of range")?;
        let value = jn(order_int, argument);
        if !value.is_finite() {
            return Err(BcError::Error(
                "Bessel function produced a non-finite result".to_string(),
            ));
        }
        Self::decimal_from_f64_static(value, "Failed to convert bessel result")
    }

    fn builtin_rand(&mut self, args: &[Decimal]) -> Result<Decimal, BcError> {
        match args.len() {
            0 => Ok(Decimal::from((self.rng.next_u32() & 0x7fff) as i64)),
            1 => {
                let limit = args[0].floor();
                if limit.sign() != Sign::Positive {
                    return Err(BcError::Error("rand(n) expects n > 0".to_string()));
                }
                let upper = ToPrimitive::to_u32(&limit).ok_or_else(|| {
                    BcError::Error("rand(n) limit is out of range".to_string())
                })?;
                if upper == 0 {
                    return Ok(Decimal::ZERO);
                }
                Ok(Decimal::from(self.rng.gen_range(0..upper) as i64))
            }
            n => Err(BcError::Error(format!(
                "rand() expects 0 or 1 arguments, got {n}",
            ))),
        }
    }

    fn builtin_srand(&mut self, args: &[Decimal]) -> Result<Decimal, BcError> {
        if args.len() != 1 {
            return Err(BcError::Error("srand(seed) expects exactly 1 argument".to_string()));
        }
        let seed_val = args[0].trunc();
        if seed_val.sign() == Sign::Negative {
            return Err(BcError::Error("srand(seed) expects non-negative seed".to_string()));
        }
        let seed = ToPrimitive::to_u64(&seed_val).ok_or_else(|| {
            BcError::Error("srand(seed) out of range".to_string())
        })?;
        self.rng = SmallRng::seed_from_u64(seed);
        Ok(Decimal::from(seed))
    }

    fn builtin_math_unary(
        name: &str,
        args: &[Decimal],
        func: impl Fn(f64) -> f64,
    ) -> Result<Decimal, BcError> {
        if args.len() != 1 {
            return Err(BcError::Error(format!(
                "{}() expects 1 argument, got {}",
                name,
                args.len()
            )));
        }
        let input = Self::decimal_to_f64(&args[0], &format!("{name}() argument out of range"))?;
        let result = func(input);
        Self::decimal_from_f64_static(result, &format!("{name}() produced invalid result"))
    }

    fn builtin_math_binary(
        name: &str,
        args: &[Decimal],
        func: impl Fn(f64, f64) -> f64,
    ) -> Result<Decimal, BcError> {
        if args.len() != 2 {
            return Err(BcError::Error(format!(
                "{}() expects 2 arguments, got {}",
                name,
                args.len()
            )));
        }
        let lhs = Self::decimal_to_f64(&args[0], &format!("{name}() argument out of range"))?;
        let rhs = Self::decimal_to_f64(&args[1], &format!("{name}() argument out of range"))?;
        let result = func(lhs, rhs);
        Self::decimal_from_f64_static(result, &format!("{name}() produced invalid result"))
    }

    fn builtin_decimal_unary(
        name: &str,
        args: &[Decimal],
        func: impl Fn(&Decimal) -> Decimal,
    ) -> Result<Decimal, BcError> {
        if args.len() != 1 {
            return Err(BcError::Error(format!(
                "{}() expects 1 argument, got {}",
                name,
                args.len()
            )));
        }
        Ok(func(&args[0]))
    }

    fn builtin_abs(args: &[Decimal]) -> Result<Decimal, BcError> {
        if args.len() != 1 {
            return Err(BcError::Error(format!(
                "abs() expects 1 argument, got {}",
                args.len()
            )));
        }
        Ok(args[0].clone().abs())
    }

    fn builtin_log(args: &[Decimal]) -> Result<Decimal, BcError> {
        match args.len() {
            1 => {
                let value = Self::decimal_to_f64(&args[0], "log() expects positive input")?;
                if value <= 0.0 {
                    return Err(BcError::Error("log() expects positive input".to_string()));
                }
                let result = libm::log10(value);
                Self::decimal_from_f64_static(result, "log() produced invalid result")
            }
            2 => {
                let base = Self::decimal_to_f64(&args[0], "log() base out of range")?;
                let value = Self::decimal_to_f64(&args[1], "log() expects positive argument")?;
                if value <= 0.0 {
                    return Err(BcError::Error("log() expects positive argument".to_string()));
                }
                if base <= 0.0 || (base - 1.0).abs() < f64::EPSILON {
                    return Err(BcError::Error(
                        "log() base must be positive and not equal to 1".to_string(),
                    ));
                }
                let result = libm::log(value) / libm::log(base);
                Self::decimal_from_f64_static(result, "log() produced invalid result")
            }
            n => Err(BcError::Error(format!(
                "log() expects 1 or 2 arguments, got {n}",
            ))),
        }
    }

    fn builtin_pow(args: &[Decimal]) -> Result<Decimal, BcError> {
        if args.len() != 2 {
            return Err(BcError::Error("pow() expects 2 arguments".to_string()));
        }
        let base = &args[0];
        let exponent = &args[1];
        if exponent.fract().is_zero() {
            let power = ToPrimitive::to_i64(&exponent.trunc()).ok_or_else(|| {
                BcError::Error("pow() exponent out of range".to_string())
            })?;
            Ok(base.clone().powi(power.into()))
        } else {
            let base_f = Self::decimal_to_f64(base, "pow() base out of range")?;
            let exp_f = Self::decimal_to_f64(exponent, "pow() exponent out of range")?;
            let result = libm::pow(base_f, exp_f);
            Self::decimal_from_f64_static(result, "pow() produced invalid result")
        }
    }

    fn builtin_sign(args: &[Decimal]) -> Result<Decimal, BcError> {
        if args.len() != 1 {
            return Err(BcError::Error(format!(
                "sign() expects 1 argument, got {}",
                args.len()
            )));
        }
        Ok(Self::decimal_sign(&args[0]))
    }

    fn builtin_min(args: &[Decimal]) -> Result<Decimal, BcError> {
        if args.len() < 2 {
            return Err(BcError::Error("min() expects at least 2 arguments".to_string()));
        }
        let mut current = args[0].clone();
        for value in &args[1..] {
            if value < &current {
                current = value.clone();
            }
        }
        Ok(current)
    }

    fn builtin_max(args: &[Decimal]) -> Result<Decimal, BcError> {
        if args.len() < 2 {
            return Err(BcError::Error("max() expects at least 2 arguments".to_string()));
        }
        let mut current = args[0].clone();
        for value in &args[1..] {
            if value > &current {
                current = value.clone();
            }
        }
        Ok(current)
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
            return self.format_result_decimal(&value);
        }
        if let Some(formatted) = self.format_result_obase(&value) {
            formatted
        } else {
            self.format_result_decimal(&value)
        }
    }

    fn decimal_from_f64(&self, value: f64, err: &str) -> Result<Decimal, BcError> {
        let decimal = Self::decimal_from_f64_static(value, err)?;
        Ok(self.promote_precision(decimal))
    }

    fn decimal_from_f64_static(value: f64, err: &str) -> Result<Decimal, BcError> {
        value
            .to_string()
            .parse::<Decimal>()
            .map_err(|_| BcError::Error(err.to_string()))
    }

    fn substitute_numeric_literals(&mut self, expr: &str) -> Result<String, BcError> {
        let mut result = String::with_capacity(expr.len());
        let chars: Vec<char> = expr.chars().collect();
        let mut index = 0;
        while index < chars.len() {
            if let Some((literal, consumed)) = Self::extract_numeric_literal(&chars, index) {
                let name = self.next_literal_name();
                let decimal = literal
                    .parse::<Decimal>()
                    .map_err(|_| BcError::Error(format!("Failed to parse literal: {literal}")))?;
                self.literal_values.insert(name.clone(), decimal);
                result.push_str(&name);
                index += consumed;
            } else {
                result.push(chars[index]);
                index += 1;
            }
        }
        Ok(result)
    }

    fn extract_numeric_literal(chars: &[char], start: usize) -> Option<(String, usize)> {
        let len = chars.len();
        let mut index = start;
        let mut literal = String::new();

        let prev = Self::previous_non_whitespace(chars, start);

        if index >= len {
            return None;
        }

        // Optional sign for unary literals
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

        if chars[index].is_ascii_digit() {
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
                // A trailing decimal point without digits is not a literal
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

        // Ensure the literal is not immediately followed by an identifier character
        if index < len {
            let next = chars[index];
            if next.is_ascii_alphanumeric() || next == '_' {
                return None;
            }
            if next == '.' && !has_decimal_point {
                // Cases like "1." should be treated as literal, but "1.x" should not
                if index + 1 < len && chars[index + 1].is_ascii_alphabetic() {
                    return None;
                }
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

    fn next_literal_name(&mut self) -> String {
        let name = format!("__dntk_lit{}", self.literal_counter);
        self.literal_counter = self.literal_counter.wrapping_add(1);
        name
    }

    fn promote_precision(&self, value: Decimal) -> Decimal {
        const PRECISION_PADDING: usize = 4;
        let digits = value.repr().digits().max(1);
        let target = digits
            .saturating_mul(2)
            .saturating_add(self.scale as usize)
            .saturating_add(PRECISION_PADDING)
            .max(1);
        match value.with_precision(target) {
            Approximation::Exact(v) | Approximation::Inexact(v, _) => v,
        }
    }

    fn truncate_decimal_to_scale(value: &Decimal, scale: u32) -> Decimal {
        if scale == 0 {
            return value.trunc();
        }

        let factor = Decimal::from(10).powi(scale.into());
        let truncated = (value.clone() * &factor).trunc();
        truncated / factor
    }

    fn decimal_to_plain_string(value: &Decimal) -> String {
        let repr = value.repr();
        if repr.significand().is_zero() {
            return "0".to_string();
        }

        let mut digits = repr.significand().unsigned_abs().to_string();
        let exponent = repr.exponent();

        if exponent >= 0 {
            digits.extend(std::iter::repeat('0').take(exponent as usize));
        } else {
            let shift = (-exponent) as usize;
            if digits.len() <= shift {
                let mut buffer = String::from("0.");
                buffer.extend(std::iter::repeat('0').take(shift - digits.len()));
                buffer.push_str(&digits);
                digits = buffer;
            } else {
                let split = digits.len() - shift;
                let (int_part, frac_part) = digits.split_at(split);
                let mut buffer = int_part.to_string();
                buffer.push('.');
                buffer.push_str(frac_part);
                digits = buffer;
            }
        }

        if repr.sign() == Sign::Negative {
            format!("-{}", digits)
        } else {
            digits
        }
    }

    fn format_result_decimal(&self, value: &Decimal) -> String {
        let scale = self.scale;
        let truncated = Self::truncate_decimal_to_scale(value, scale);
        let mut formatted = Self::decimal_to_plain_string(&truncated);

        if let Some(point_index) = formatted.find('.') {
            if scale == 0 {
                formatted.truncate(point_index);
            } else {
                let frac_len = formatted.len() - point_index - 1;
                if frac_len < scale as usize {
                    formatted
                        .extend(std::iter::repeat('0').take(scale as usize - frac_len));
                }
            }
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

    fn format_result_obase(&self, value: &Decimal) -> Option<String> {
        const DIGITS: &[u8; 36] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let base = self.obase as i32;
        let negative = value.sign() == Sign::Negative;
        let abs_value = value.clone().abs();
        let integer_part = abs_value.trunc();
        let mut integer = ToPrimitive::to_i128(&integer_part)?;

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
                fraction *= base_decimal.clone();
                let digit_dec = fraction.trunc();
                let digit = ToPrimitive::to_u32(&digit_dec)? as usize;
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
