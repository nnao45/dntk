use std::str::FromStr;

use dashu::Decimal;

use super::complex::{ComplexNumber, ComplexParseError};
use super::error::BcError;

impl super::BcExecuter {
    pub(super) fn try_eval_matrix_expression(
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

        let (value, seen_matrix) = match MatrixExpression::parse(self, trimmed) {
            Ok(result) => result,
            Err(MatrixParseError::NotMatrix) => return Ok(None),
            Err(MatrixParseError::Invalid(msg)) => return Err(BcError::Error(msg)),
        };

        if !seen_matrix {
            return Ok(None);
        }

        match value {
            MatrixValue::Matrix(matrix) => {
                let promoted = self.promote_matrix_precision(matrix);
                Ok(Some(self.format_matrix(&promoted)))
            }
            MatrixValue::Scalar(_) => Err(BcError::Error(
                "matrix expression must evaluate to a matrix".to_string(),
            )),
        }
    }
}

struct MatrixExpression;

impl MatrixExpression {
    fn parse(
        exec: &super::BcExecuter,
        expr: &str,
    ) -> Result<(MatrixValue, bool), MatrixParseError> {
        let (tokens, seen_matrix) = MatrixLexer::tokenize(exec, expr)?;
        if tokens.is_empty() {
            return Err(MatrixParseError::NotMatrix);
        }
        if !seen_matrix {
            return Err(MatrixParseError::NotMatrix);
        }
        let mut parser = MatrixParser::new(exec, tokens);
        let value = parser.parse_expression()?;
        parser.expect_end()?;
        Ok((value, seen_matrix))
    }
}

struct MatrixParser<'a> {
    tokens: Vec<MatrixToken>,
    position: usize,
    exec: &'a super::BcExecuter,
}

impl<'a> MatrixParser<'a> {
    fn new(exec: &'a super::BcExecuter, tokens: Vec<MatrixToken>) -> Self {
        Self {
            tokens,
            position: 0,
            exec,
        }
    }

    fn parse_expression(&mut self) -> Result<MatrixValue, MatrixParseError> {
        let mut value = self.parse_term()?;
        while let Some(token) = self.peek() {
            match token {
                MatrixToken::Plus => {
                    self.next();
                    let rhs = self.parse_term()?;
                    value = value.add(rhs)?;
                }
                MatrixToken::Minus => {
                    self.next();
                    let rhs = self.parse_term()?;
                    value = value.sub(rhs)?;
                }
                _ => break,
            }
        }
        Ok(value)
    }

    fn parse_term(&mut self) -> Result<MatrixValue, MatrixParseError> {
        let mut value = self.parse_unary()?;
        while let Some(token) = self.peek() {
            match token {
                MatrixToken::Star => {
                    self.next();
                    let rhs = self.parse_unary()?;
                    value = value.mul(rhs)?;
                }
                MatrixToken::Slash => {
                    self.next();
                    let rhs = self.parse_unary()?;
                    value = value.div(rhs)?;
                }
                _ => break,
            }
        }
        Ok(value)
    }

    fn parse_unary(&mut self) -> Result<MatrixValue, MatrixParseError> {
        if let Some(token) = self.peek() {
            match token {
                MatrixToken::Plus => {
                    self.next();
                    return self.parse_unary();
                }
                MatrixToken::Minus => {
                    self.next();
                    let value = self.parse_unary()?;
                    return Ok(value.negate());
                }
                MatrixToken::Function(function) => {
                    self.next();
                    if !matches!(self.next(), Some(MatrixToken::LParen)) {
                        return Err(MatrixParseError::invalid(
                            "matrix function requires parentheses",
                        ));
                    }
                    let value = self.parse_expression()?;
                    if !matches!(self.next(), Some(MatrixToken::RParen)) {
                        return Err(MatrixParseError::invalid("mismatched parentheses"));
                    }
                    return self.apply_function(function, value);
                }
                _ => {}
            }
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<MatrixValue, MatrixParseError> {
        match self.next() {
            Some(MatrixToken::Matrix(matrix)) => Ok(MatrixValue::Matrix(matrix)),
            Some(MatrixToken::Scalar(value)) => Ok(MatrixValue::Scalar(value)),
            Some(MatrixToken::LParen) => {
                let value = self.parse_expression()?;
                if !matches!(self.next(), Some(MatrixToken::RParen)) {
                    return Err(MatrixParseError::invalid("mismatched parentheses"));
                }
                Ok(value)
            }
            _ => Err(MatrixParseError::invalid(
                "unexpected token in matrix expression",
            )),
        }
    }

    fn apply_function(
        &self,
        function: MatrixFunction,
        value: MatrixValue,
    ) -> Result<MatrixValue, MatrixParseError> {
        match function {
            MatrixFunction::Sin => self.apply_sin(value),
        }
    }

    fn apply_sin(&self, value: MatrixValue) -> Result<MatrixValue, MatrixParseError> {
        match value {
            MatrixValue::Matrix(matrix) => {
                let mut result = Vec::with_capacity(matrix.len());
                for row in matrix {
                    let mut formatted_row = Vec::with_capacity(row.len());
                    for entry in row {
                        let (real_f, imag_f) = entry
                            .sin_components()
                            .map_err(MatrixParseError::from_complex_error)?;
                        let real = self
                            .exec
                            .decimal_from_f64(real_f, "matrix sin overflowed")
                            .map_err(MatrixParseError::from_bc_error)?;
                        let imag = self
                            .exec
                            .decimal_from_f64(imag_f, "matrix sin overflowed")
                            .map_err(MatrixParseError::from_bc_error)?;
                        formatted_row.push(ComplexNumber::new(real, imag));
                    }
                    result.push(formatted_row);
                }
                Ok(MatrixValue::Matrix(result))
            }
            MatrixValue::Scalar(value) => {
                let (real_f, imag_f) = value
                    .sin_components()
                    .map_err(MatrixParseError::from_complex_error)?;
                let real = self
                    .exec
                    .decimal_from_f64(real_f, "matrix sin overflowed")
                    .map_err(MatrixParseError::from_bc_error)?;
                let imag = self
                    .exec
                    .decimal_from_f64(imag_f, "matrix sin overflowed")
                    .map_err(MatrixParseError::from_bc_error)?;
                Ok(MatrixValue::Scalar(ComplexNumber::new(real, imag)))
            }
        }
    }

    fn peek(&self) -> Option<MatrixToken> {
        self.tokens.get(self.position).cloned()
    }

    fn next(&mut self) -> Option<MatrixToken> {
        let token = self.tokens.get(self.position).cloned();
        if token.is_some() {
            self.position += 1;
        }
        token
    }

    fn expect_end(&mut self) -> Result<(), MatrixParseError> {
        if self.position == self.tokens.len() {
            Ok(())
        } else {
            Err(MatrixParseError::invalid("unexpected trailing tokens"))
        }
    }
}

#[derive(Clone, Copy)]
enum MatrixFunction {
    Sin,
}

#[derive(Clone)]
enum MatrixToken {
    Matrix(Vec<Vec<ComplexNumber>>),
    Scalar(ComplexNumber),
    Function(MatrixFunction),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

struct MatrixLexer;

impl MatrixLexer {
    fn tokenize(
        exec: &super::BcExecuter,
        expr: &str,
    ) -> Result<(Vec<MatrixToken>, bool), MatrixParseError> {
        let mut tokens = Vec::new();
        let mut remaining = expr.trim_start();
        let mut seen_matrix = false;
        while !remaining.is_empty() {
            let ch = remaining.chars().next().unwrap();
            match ch {
                '[' => {
                    let (literal, rest) = super::BcExecuter::extract_matrix_literal(remaining)
                        .map_err(MatrixParseError::from_bc_error)?;
                    let matrix = exec
                        .parse_matrix(literal)
                        .map_err(MatrixParseError::from_bc_error)?;
                    let matrix = exec.promote_matrix_precision(matrix);
                    tokens.push(MatrixToken::Matrix(matrix));
                    seen_matrix = true;
                    remaining = rest;
                }
                '+' => {
                    tokens.push(MatrixToken::Plus);
                    remaining = &remaining[ch.len_utf8()..];
                }
                '-' => {
                    tokens.push(MatrixToken::Minus);
                    remaining = &remaining[ch.len_utf8()..];
                }
                '*' => {
                    tokens.push(MatrixToken::Star);
                    remaining = &remaining[ch.len_utf8()..];
                }
                '/' => {
                    tokens.push(MatrixToken::Slash);
                    remaining = &remaining[ch.len_utf8()..];
                }
                '(' => {
                    tokens.push(MatrixToken::LParen);
                    remaining = &remaining[ch.len_utf8()..];
                }
                ')' => {
                    tokens.push(MatrixToken::RParen);
                    remaining = &remaining[ch.len_utf8()..];
                }
                '0'..='9' | '.' => {
                    let (number, rest) = MatrixLexer::parse_number(remaining)?;
                    let promoted = exec.promote_precision(number.clone());
                    let mut scalar = ComplexNumber::from_real(promoted);
                    let mut remainder = rest;
                    if remainder.starts_with('i') {
                        if MatrixLexer::is_identifier_continuation(remainder, 'i'.len_utf8()) {
                            return Err(MatrixParseError::NotMatrix);
                        }
                        let imag = exec.promote_precision(number);
                        let zero = exec.promote_precision(Decimal::ZERO);
                        scalar = ComplexNumber::new(zero, imag);
                        remainder = &remainder['i'.len_utf8()..];
                    }
                    tokens.push(MatrixToken::Scalar(scalar));
                    remaining = remainder;
                }
                _ => {
                    if ch.is_alphabetic() {
                        let (ident, rest) = MatrixLexer::parse_identifier(remaining);
                        if let Some(function) = MatrixLexer::map_function(&ident) {
                            tokens.push(MatrixToken::Function(function));
                            remaining = rest;
                        } else {
                            return Err(MatrixParseError::NotMatrix);
                        }
                    } else {
                        if seen_matrix {
                            return Err(MatrixParseError::invalid(
                                "unsupported character in matrix expression",
                            ));
                        }
                        return Err(MatrixParseError::NotMatrix);
                    }
                }
            }
            remaining = remaining.trim_start();
        }
        Ok((tokens, seen_matrix))
    }

    fn parse_number(input: &str) -> Result<(Decimal, &str), MatrixParseError> {
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
            return Err(MatrixParseError::NotMatrix);
        }
        let number_str = &input[..len];
        let number = Decimal::from_str(number_str)
            .map_err(|_| MatrixParseError::invalid("failed to parse matrix number"))?;
        Ok((number, &input[len..]))
    }

    fn parse_identifier(input: &str) -> (String, &str) {
        let mut len = 0usize;
        for ch in input.chars() {
            if ch.is_ascii_alphabetic() {
                len += ch.len_utf8();
            } else {
                break;
            }
        }
        let ident = input[..len].to_string();
        (ident, &input[len..])
    }

    fn map_function(name: &str) -> Option<MatrixFunction> {
        match name {
            "sin" => Some(MatrixFunction::Sin),
            _ => None,
        }
    }

    fn is_identifier_continuation(input: &str, offset: usize) -> bool {
        if offset >= input.len() {
            return false;
        }
        input[offset..]
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
    }
}

#[derive(Clone)]
enum MatrixValue {
    Matrix(Vec<Vec<ComplexNumber>>),
    Scalar(ComplexNumber),
}

impl MatrixValue {
    fn add(self, other: MatrixValue) -> Result<Self, MatrixParseError> {
        match (self, other) {
            (MatrixValue::Matrix(lhs), MatrixValue::Matrix(rhs)) => {
                let result = super::BcExecuter::matrix_add(&lhs, &rhs)
                    .map_err(MatrixParseError::from_bc_error)?;
                Ok(MatrixValue::Matrix(result))
            }
            (MatrixValue::Scalar(lhs), MatrixValue::Scalar(rhs)) => {
                Ok(MatrixValue::Scalar(lhs.add(&rhs)))
            }
            _ => Err(MatrixParseError::invalid(
                "matrix addition requires matching types",
            )),
        }
    }

    fn sub(self, other: MatrixValue) -> Result<Self, MatrixParseError> {
        match (self, other) {
            (MatrixValue::Matrix(lhs), MatrixValue::Matrix(rhs)) => {
                let result = super::BcExecuter::matrix_sub(&lhs, &rhs)
                    .map_err(MatrixParseError::from_bc_error)?;
                Ok(MatrixValue::Matrix(result))
            }
            (MatrixValue::Scalar(lhs), MatrixValue::Scalar(rhs)) => {
                Ok(MatrixValue::Scalar(lhs.sub(&rhs)))
            }
            _ => Err(MatrixParseError::invalid(
                "matrix subtraction requires matching types",
            )),
        }
    }

    fn mul(self, other: MatrixValue) -> Result<Self, MatrixParseError> {
        match (self, other) {
            (MatrixValue::Matrix(lhs), MatrixValue::Matrix(rhs)) => {
                let result = super::BcExecuter::matrix_mul(&lhs, &rhs)
                    .map_err(MatrixParseError::from_bc_error)?;
                Ok(MatrixValue::Matrix(result))
            }
            (MatrixValue::Matrix(matrix), MatrixValue::Scalar(scalar)) => {
                Ok(MatrixValue::Matrix(scale_matrix(matrix, &scalar)))
            }
            (MatrixValue::Scalar(scalar), MatrixValue::Matrix(matrix)) => {
                Ok(MatrixValue::Matrix(scale_matrix(matrix, &scalar)))
            }
            (MatrixValue::Scalar(lhs), MatrixValue::Scalar(rhs)) => {
                Ok(MatrixValue::Scalar(lhs.mul(&rhs)))
            }
        }
    }

    fn div(self, other: MatrixValue) -> Result<Self, MatrixParseError> {
        match (self, other) {
            (MatrixValue::Matrix(matrix), MatrixValue::Scalar(scalar)) => {
                let result = divide_matrix(matrix, &scalar)?;
                Ok(MatrixValue::Matrix(result))
            }
            (MatrixValue::Scalar(lhs), MatrixValue::Scalar(rhs)) => {
                let result = lhs
                    .div(&rhs)
                    .map_err(MatrixParseError::from_complex_error)?;
                Ok(MatrixValue::Scalar(result))
            }
            _ => Err(MatrixParseError::invalid(
                "matrix division requires a scalar divisor",
            )),
        }
    }

    fn negate(self) -> Self {
        match self {
            MatrixValue::Matrix(matrix) => {
                let neg_one = ComplexNumber::new(Decimal::from(-1), Decimal::ZERO);
                MatrixValue::Matrix(scale_matrix(matrix, &neg_one))
            }
            MatrixValue::Scalar(value) => MatrixValue::Scalar(value.negate()),
        }
    }
}

fn scale_matrix(
    matrix: Vec<Vec<ComplexNumber>>,
    scalar: &ComplexNumber,
) -> Vec<Vec<ComplexNumber>> {
    matrix
        .into_iter()
        .map(|row| row.into_iter().map(|value| value.mul(scalar)).collect())
        .collect()
}

fn divide_matrix(
    matrix: Vec<Vec<ComplexNumber>>,
    scalar: &ComplexNumber,
) -> Result<Vec<Vec<ComplexNumber>>, MatrixParseError> {
    if scalar.is_zero() {
        return Err(MatrixParseError::invalid("matrix scalar division by zero"));
    }
    let mut result = Vec::with_capacity(matrix.len());
    for row in matrix {
        let mut divided = Vec::with_capacity(row.len());
        for value in row {
            let quotient = value
                .div(scalar)
                .map_err(MatrixParseError::from_complex_error)?;
            divided.push(quotient);
        }
        result.push(divided);
    }
    Ok(result)
}

#[derive(Debug)]
enum MatrixParseError {
    NotMatrix,
    Invalid(String),
}

impl MatrixParseError {
    fn invalid(message: &str) -> Self {
        MatrixParseError::Invalid(message.to_string())
    }

    fn from_bc_error(error: BcError) -> Self {
        match error {
            BcError::NoResult => {
                MatrixParseError::Invalid("matrix evaluation did not produce a result".to_string())
            }
            BcError::Error(msg) => MatrixParseError::Invalid(msg),
        }
    }

    fn from_complex_error(error: ComplexParseError) -> Self {
        MatrixParseError::Invalid(error.into_message())
    }
}
