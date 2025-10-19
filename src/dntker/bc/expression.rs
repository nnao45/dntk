use std::collections::BTreeMap;

use dashu::base::{Abs, Sign};
use dashu::Decimal;
use fasteval::compiler::{Compiler, Instruction, InstructionI, IC};
use fasteval::slab::CompileSlab;
use fasteval::Slab;
use libm::jn;
use num_traits::{ToPrimitive, Zero};
use rand::{Rng, RngCore};

use super::error::BcError;
use super::runtime::StatementOutcome;

impl super::BcExecuter {
    pub(super) fn eval_expression(&mut self, expr: &str) -> Result<Decimal, BcError> {
        let trimmed = expr.trim();

        if let Ok(decimal_value) = trimmed.parse::<Decimal>() {
            return Ok(self.promote_precision(decimal_value));
        }

        self.literals.reset();
        let processed = self.preprocess_bc_syntax(expr);
        let substituted = self.literals.substitute(&processed)?;
        let mut slab = Slab::new();
        let expr_idx = self
            .parser
            .parse(&substituted, &mut slab.ps)
            .map_err(|e| BcError::Error(e.to_string()))?;
        let expression = expr_idx.from(&slab.ps);
        let instruction = expression.compile(&slab.ps, &mut slab.cs);
        let value = self.eval_instruction(&instruction, &slab.cs)?;
        self.literals.reset();
        Ok(self.promote_precision(value))
    }

    fn eval_instruction(
        &mut self,
        instruction: &Instruction,
        compile_slab: &CompileSlab,
    ) -> Result<Decimal, BcError> {
        let value = match instruction {
            Instruction::IConst(value) => {
                self.decimal_from_f64(*value, "Failed to convert constant")
            }
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
                    return Err(BcError::Error(
                        "round() expects non-zero modulus".to_string(),
                    ));
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
            Instruction::IFuncSin(idx) => {
                self.eval_math_function(*idx, compile_slab, libm::sin, "sin")
            }
            Instruction::IFuncCos(idx) => {
                self.eval_math_function(*idx, compile_slab, libm::cos, "cos")
            }
            Instruction::IFuncTan(idx) => {
                self.eval_math_function(*idx, compile_slab, libm::tan, "tan")
            }
            Instruction::IFuncASin(idx) => {
                self.eval_math_function(*idx, compile_slab, libm::asin, "asin")
            }
            Instruction::IFuncACos(idx) => {
                self.eval_math_function(*idx, compile_slab, libm::acos, "acos")
            }
            Instruction::IFuncATan(idx) => {
                self.eval_math_function(*idx, compile_slab, libm::atan, "atan")
            }
            Instruction::IFuncSinH(idx) => {
                self.eval_math_function(*idx, compile_slab, libm::sinh, "sinh")
            }
            Instruction::IFuncCosH(idx) => {
                self.eval_math_function(*idx, compile_slab, libm::cosh, "cosh")
            }
            Instruction::IFuncTanH(idx) => {
                self.eval_math_function(*idx, compile_slab, libm::tanh, "tanh")
            }
            Instruction::IFuncASinH(idx) => {
                self.eval_math_function(*idx, compile_slab, libm::asinh, "asinh")
            }
            Instruction::IFuncACosH(idx) => {
                self.eval_math_function(*idx, compile_slab, libm::acosh, "acosh")
            }
            Instruction::IFuncATanH(idx) => {
                self.eval_math_function(*idx, compile_slab, libm::atanh, "atanh")
            }
            Instruction::IPrintFunc(_) => {
                Err(BcError::Error("print() is not supported".to_string()))
            }
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
            let power = ToPrimitive::to_i64(&exponent.trunc())
                .ok_or_else(|| BcError::Error("Exponent out of supported range".to_string()))?;
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
            return Err(BcError::Error(
                "log() expects positive argument".to_string(),
            ));
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
            if let Some(value) = self.literals.get(name) {
                return Ok(value);
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
        self.runtime.get_variable(name)
    }

    fn call_function(
        &mut self,
        name: &str,
        args: Vec<Decimal>,
    ) -> Result<Option<Decimal>, BcError> {
        let def = match self.runtime.get_function(name) {
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

        self.runtime.push_scope(local_scope);
        let outcome = self.eval_block(def.body)?;
        self.runtime.pop_scope();

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
            0 => {
                let rng = self.runtime.rng_mut();
                Ok(Decimal::from((rng.next_u32() & 0x7fff) as i64))
            }
            1 => {
                let limit = args[0].floor();
                if limit.sign() != Sign::Positive {
                    return Err(BcError::Error("rand(n) expects n > 0".to_string()));
                }
                let upper = ToPrimitive::to_u32(&limit)
                    .ok_or_else(|| BcError::Error("rand(n) limit is out of range".to_string()))?;
                if upper == 0 {
                    return Ok(Decimal::ZERO);
                }
                let rng = self.runtime.rng_mut();
                Ok(Decimal::from(rng.gen_range(0..upper) as i64))
            }
            n => Err(BcError::Error(format!(
                "rand() expects 0 or 1 arguments, got {n}",
            ))),
        }
    }

    fn builtin_srand(&mut self, args: &[Decimal]) -> Result<Decimal, BcError> {
        if args.len() != 1 {
            return Err(BcError::Error(
                "srand(seed) expects exactly 1 argument".to_string(),
            ));
        }
        let seed_val = args[0].trunc();
        if seed_val.sign() == Sign::Negative {
            return Err(BcError::Error(
                "srand(seed) expects non-negative seed".to_string(),
            ));
        }
        let seed = ToPrimitive::to_u64(&seed_val)
            .ok_or_else(|| BcError::Error("srand(seed) out of range".to_string()))?;
        self.runtime.reseed_rng(seed);
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
                    return Err(BcError::Error(
                        "log() expects positive argument".to_string(),
                    ));
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
            let power = ToPrimitive::to_i64(&exponent.trunc())
                .ok_or_else(|| BcError::Error("pow() exponent out of range".to_string()))?;
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
            return Err(BcError::Error(
                "min() expects at least 2 arguments".to_string(),
            ));
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
            return Err(BcError::Error(
                "max() expects at least 2 arguments".to_string(),
            ));
        }
        let mut current = args[0].clone();
        for value in &args[1..] {
            if value > &current {
                current = value.clone();
            }
        }
        Ok(current)
    }
}
