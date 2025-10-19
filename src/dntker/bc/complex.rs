use std::str::FromStr;

use dashu::Decimal;
use libm::{cos, cosh, sin, sinh, sqrt};
use num_traits::{ToPrimitive, Zero};

use super::error::BcError;

impl super::BcExecuter {
    pub(super) fn try_eval_complex_expression(
        &mut self,
        statement: &str,
    ) -> Result<Option<String>, BcError> {
        let trimmed = statement.trim();
        if trimmed.is_empty()
            || trimmed.contains('=')
            || trimmed.contains('{')
            || trimmed.contains('}')
        {
            return Ok(None);
        }

        if let Some(inner) = Self::strip_wrapped_function(trimmed, "abs")? {
            let parsed = match ComplexExpression::parse(self, inner) {
                Ok(parsed) => parsed,
                Err(ComplexParseError::NotComplex) => return Ok(None),
                Err(ComplexParseError::Invalid(msg)) => return Err(BcError::Error(msg)),
            };
            let magnitude = match parsed.value.magnitude() {
                Ok(value) => value,
                Err(ComplexParseError::Invalid(msg)) => return Err(BcError::Error(msg)),
                Err(ComplexParseError::NotComplex) => unreachable!(),
            };
            let decimal = self.decimal_from_f64(magnitude, "complex abs overflowed")?;
            return Ok(Some(self.format_result_decimal(&decimal)));
        }

        if let Some(inner) = Self::strip_wrapped_function(trimmed, "sin")? {
            let parsed = match ComplexExpression::parse(self, inner) {
                Ok(parsed) => parsed,
                Err(ComplexParseError::NotComplex) => return Ok(None),
                Err(ComplexParseError::Invalid(msg)) => return Err(BcError::Error(msg)),
            };
            let (real_f, imag_f) = parsed
                .value
                .sin_components()
                .map_err(|err| BcError::Error(err.into_message()))?;
            let real = self.decimal_from_f64(real_f, "complex sin overflowed")?;
            let imag = self.decimal_from_f64(imag_f, "complex sin overflowed")?;
            return Ok(Some(self.format_complex_result(real, imag)));
        }

        let parsed = match ComplexExpression::parse(self, trimmed) {
            Ok(parsed) => parsed,
            Err(ComplexParseError::NotComplex) => return Ok(None),
            Err(ComplexParseError::Invalid(msg)) => return Err(BcError::Error(msg)),
        };
        let real = self.promote_precision(parsed.value.real);
        let imag = self.promote_precision(parsed.value.imag);
        Ok(Some(self.format_complex_result(real, imag)))
    }

    pub(super) fn parse_complex_literal(
        &self,
        expr: &str,
    ) -> Result<Option<ComplexNumber>, ComplexParseError> {
        let trimmed = expr.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }
        match ComplexExpression::parse(self, trimmed) {
            Ok(parsed) => Ok(Some(parsed.value)),
            Err(ComplexParseError::NotComplex) => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn strip_wrapped_function<'a>(input: &'a str, name: &str) -> Result<Option<&'a str>, BcError> {
        let trimmed = input.trim();
        if !trimmed.starts_with(name) {
            return Ok(None);
        }
        let after_name = trimmed[name.len()..].trim_start();
        if !after_name.starts_with('(') {
            return Ok(None);
        }
        let closing = Self::find_matching(after_name, 0, '(', ')')?;
        if closing + 1 != after_name.len() {
            return Ok(None);
        }
        Ok(Some(after_name[1..closing].trim()))
    }
}

struct ComplexExpression {
    value: ComplexNumber,
}

impl ComplexExpression {
    fn parse(exec: &super::BcExecuter, expr: &str) -> Result<Self, ComplexParseError> {
        let (tokens, seen_imaginary, saw_number) = ComplexLexer::tokenize(expr)?;
        if tokens.is_empty() || !seen_imaginary || !saw_number {
            return Err(ComplexParseError::NotComplex);
        }
        let mut parser = ComplexParser::new(exec, tokens);
        let value = parser.parse_expression()?;
        parser.expect_end()?;
        Ok(ComplexExpression { value })
    }
}

struct ComplexParser<'a> {
    tokens: Vec<ComplexToken>,
    position: usize,
    exec: &'a super::BcExecuter,
}

impl<'a> ComplexParser<'a> {
    fn new(exec: &'a super::BcExecuter, tokens: Vec<ComplexToken>) -> Self {
        Self {
            tokens,
            position: 0,
            exec,
        }
    }

    fn parse_expression(&mut self) -> Result<ComplexNumber, ComplexParseError> {
        let mut value = self.parse_term()?;
        while let Some(token) = self.peek() {
            match token {
                ComplexToken::Plus => {
                    self.next();
                    let rhs = self.parse_term()?;
                    value = value.add(&rhs);
                }
                ComplexToken::Minus => {
                    self.next();
                    let rhs = self.parse_term()?;
                    value = value.sub(&rhs);
                }
                _ => break,
            }
        }
        Ok(value)
    }

    fn parse_term(&mut self) -> Result<ComplexNumber, ComplexParseError> {
        let mut value = self.parse_unary()?;
        while let Some(token) = self.peek() {
            match token {
                ComplexToken::Star => {
                    self.next();
                    let rhs = self.parse_unary()?;
                    value = value.mul(&rhs);
                }
                ComplexToken::Slash => {
                    self.next();
                    let rhs = self.parse_unary()?;
                    value = value.div(&rhs)?;
                }
                _ => break,
            }
        }
        Ok(value)
    }

    fn parse_unary(&mut self) -> Result<ComplexNumber, ComplexParseError> {
        if let Some(token) = self.peek() {
            match token {
                ComplexToken::Plus => {
                    self.next();
                    return self.parse_unary();
                }
                ComplexToken::Minus => {
                    self.next();
                    let value = self.parse_unary()?;
                    return Ok(value.negate());
                }
                _ => {}
            }
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<ComplexNumber, ComplexParseError> {
        match self.next() {
            Some(ComplexToken::Number(value)) => {
                let promoted = self.exec.promote_precision(value);
                if self.peek_is_i() {
                    self.next();
                    let zero = self.exec.promote_precision(Decimal::ZERO);
                    Ok(ComplexNumber::new(zero, promoted))
                } else {
                    let zero = self.exec.promote_precision(Decimal::ZERO);
                    Ok(ComplexNumber::new(promoted, zero))
                }
            }
            Some(ComplexToken::ImaginaryUnit) => {
                let real_zero = self.exec.promote_precision(Decimal::ZERO);
                let imag_one = self.exec.promote_precision(Decimal::from(1));
                Ok(ComplexNumber::new(real_zero, imag_one))
            }
            Some(ComplexToken::LParen) => {
                let value = self.parse_expression()?;
                if !matches!(self.next(), Some(ComplexToken::RParen)) {
                    return Err(ComplexParseError::invalid("mismatched parentheses"));
                }
                Ok(value)
            }
            _ => Err(ComplexParseError::invalid(
                "unexpected token in complex expression",
            )),
        }
    }

    fn peek(&self) -> Option<ComplexToken> {
        self.tokens.get(self.position).cloned()
    }

    fn next(&mut self) -> Option<ComplexToken> {
        let token = self.tokens.get(self.position).cloned();
        if token.is_some() {
            self.position += 1;
        }
        token
    }

    fn peek_is_i(&self) -> bool {
        matches!(
            self.tokens.get(self.position),
            Some(ComplexToken::ImaginaryUnit)
        )
    }

    fn expect_end(&mut self) -> Result<(), ComplexParseError> {
        if self.position == self.tokens.len() {
            Ok(())
        } else {
            Err(ComplexParseError::invalid("unexpected trailing tokens"))
        }
    }
}

#[derive(Clone)]
enum ComplexToken {
    Number(Decimal),
    ImaginaryUnit,
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

struct ComplexLexer;

impl ComplexLexer {
    fn tokenize(expr: &str) -> Result<(Vec<ComplexToken>, bool, bool), ComplexParseError> {
        let mut tokens = Vec::new();
        let mut seen_imaginary = false;
        let mut saw_number = false;
        let mut saw_token = false;
        let mut remaining = expr.trim_start();
        while !remaining.is_empty() {
            let ch = remaining.chars().next().unwrap();
            match ch {
                '+' => {
                    tokens.push(ComplexToken::Plus);
                    remaining = &remaining[ch.len_utf8()..];
                }
                '-' => {
                    tokens.push(ComplexToken::Minus);
                    remaining = &remaining[ch.len_utf8()..];
                }
                '*' => {
                    tokens.push(ComplexToken::Star);
                    remaining = &remaining[ch.len_utf8()..];
                }
                '/' => {
                    tokens.push(ComplexToken::Slash);
                    remaining = &remaining[ch.len_utf8()..];
                }
                '(' => {
                    tokens.push(ComplexToken::LParen);
                    remaining = &remaining[ch.len_utf8()..];
                }
                ')' => {
                    tokens.push(ComplexToken::RParen);
                    remaining = &remaining[ch.len_utf8()..];
                }
                'i' => {
                    if !saw_token {
                        return Err(ComplexParseError::NotComplex);
                    }
                    if ComplexLexer::is_identifier_continuation(remaining, ch.len_utf8()) {
                        return Err(ComplexParseError::NotComplex);
                    }
                    seen_imaginary = true;
                    tokens.push(ComplexToken::ImaginaryUnit);
                    remaining = &remaining[ch.len_utf8()..];
                }
                '0'..='9' | '.' => {
                    let (number, rest) = ComplexLexer::parse_number(remaining)?;
                    tokens.push(ComplexToken::Number(number));
                    saw_number = true;
                    remaining = rest;
                }
                _ => {
                    if ch == '[' || ch == ']' {
                        return Err(ComplexParseError::NotComplex);
                    }
                    if ch.is_alphabetic() {
                        return Err(ComplexParseError::NotComplex);
                    }
                    if seen_imaginary {
                        return Err(ComplexParseError::invalid(
                            "unsupported character in complex expression",
                        ));
                    }
                    return Err(ComplexParseError::NotComplex);
                }
            }
            remaining = remaining.trim_start();
            saw_token = !tokens.is_empty();
        }
        Ok((tokens, seen_imaginary, saw_number))
    }

    fn parse_number(input: &str) -> Result<(Decimal, &str), ComplexParseError> {
        let mut len = 0usize;
        let mut has_digit = false;
        let mut has_decimal = false;
        let bytes = input.as_bytes();
        while len < bytes.len() {
            let ch = input[len..].chars().next().unwrap();
            match ch {
                '0'..='9' => {
                    has_digit = true;
                    len += ch.len_utf8();
                }
                '.' if !has_decimal => {
                    has_decimal = true;
                    len += ch.len_utf8();
                }
                'e' | 'E' => {
                    let mut idx = len + ch.len_utf8();
                    if idx >= bytes.len() {
                        break;
                    }
                    let mut exp_iter = input[idx..].chars();
                    let mut exp_len = 0usize;
                    if let Some(sign) = exp_iter.next() {
                        if sign == '+' || sign == '-' {
                            idx += sign.len_utf8();
                        }
                    }
                    while idx < bytes.len() {
                        let next = input[idx..].chars().next().unwrap();
                        if next.is_ascii_digit() {
                            idx += next.len_utf8();
                            exp_len += 1;
                        } else {
                            break;
                        }
                    }
                    if exp_len == 0 {
                        break;
                    }
                    len = idx;
                    has_digit = true;
                    break;
                }
                _ => break,
            }
        }
        if !has_digit {
            return Err(ComplexParseError::NotComplex);
        }
        let number_str = &input[..len];
        let number = Decimal::from_str(number_str)
            .map_err(|_| ComplexParseError::invalid("failed to parse complex number"))?;
        Ok((number, &input[len..]))
    }

    fn is_identifier_continuation(input: &str, offset: usize) -> bool {
        input[offset..]
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
    }
}

#[derive(Clone)]
pub(super) struct ComplexNumber {
    pub(super) real: Decimal,
    pub(super) imag: Decimal,
}

impl ComplexNumber {
    pub(super) fn new(real: Decimal, imag: Decimal) -> Self {
        Self { real, imag }
    }

    pub(super) fn from_real(real: Decimal) -> Self {
        Self {
            real,
            imag: Decimal::ZERO,
        }
    }

    pub(super) fn add(&self, other: &Self) -> Self {
        Self {
            real: self.real.clone() + other.real.clone(),
            imag: self.imag.clone() + other.imag.clone(),
        }
    }

    pub(super) fn sub(&self, other: &Self) -> Self {
        Self {
            real: self.real.clone() - other.real.clone(),
            imag: self.imag.clone() - other.imag.clone(),
        }
    }

    pub(super) fn mul(&self, other: &Self) -> Self {
        Self {
            real: self.real.clone() * other.real.clone() - self.imag.clone() * other.imag.clone(),
            imag: self.real.clone() * other.imag.clone() + self.imag.clone() * other.real.clone(),
        }
    }

    pub(super) fn div(&self, other: &Self) -> Result<Self, ComplexParseError> {
        let denom =
            other.real.clone() * other.real.clone() + other.imag.clone() * other.imag.clone();
        if denom.is_zero() {
            return Err(ComplexParseError::invalid("complex division by zero"));
        }
        let real = (self.real.clone() * other.real.clone()
            + self.imag.clone() * other.imag.clone())
            / denom.clone();
        let imag = (self.imag.clone() * other.real.clone()
            - self.real.clone() * other.imag.clone())
            / denom;
        Ok(Self { real, imag })
    }

    pub(super) fn negate(&self) -> Self {
        Self {
            real: -self.real.clone(),
            imag: -self.imag.clone(),
        }
    }

    pub(super) fn magnitude(&self) -> Result<f64, ComplexParseError> {
        let real = ToPrimitive::to_f64(&self.real)
            .ok_or_else(|| ComplexParseError::invalid("complex abs real part out of range"))?;
        let imag = ToPrimitive::to_f64(&self.imag)
            .ok_or_else(|| ComplexParseError::invalid("complex abs imaginary part out of range"))?;
        Ok(sqrt(real * real + imag * imag))
    }

    pub(super) fn sin_components(&self) -> Result<(f64, f64), ComplexParseError> {
        let real = ToPrimitive::to_f64(&self.real)
            .ok_or_else(|| ComplexParseError::invalid("complex sin real part out of range"))?;
        let imag = ToPrimitive::to_f64(&self.imag)
            .ok_or_else(|| ComplexParseError::invalid("complex sin imaginary part out of range"))?;
        let real_part = sin(real) * cosh(imag);
        let imag_part = cos(real) * sinh(imag);
        Ok((real_part, imag_part))
    }

    pub(super) fn is_zero(&self) -> bool {
        self.real.is_zero() && self.imag.is_zero()
    }
}

#[derive(Debug)]
pub(super) enum ComplexParseError {
    NotComplex,
    Invalid(String),
}

impl ComplexParseError {
    fn invalid(message: &str) -> Self {
        ComplexParseError::Invalid(message.to_string())
    }

    pub(super) fn into_message(self) -> String {
        match self {
            ComplexParseError::NotComplex => "failed to parse complex literal".to_string(),
            ComplexParseError::Invalid(msg) => msg,
        }
    }
}
