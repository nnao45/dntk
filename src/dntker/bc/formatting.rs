use std::iter;

use dashu::base::{Abs, Approximation, Sign, UnsignedAbs};
use dashu::Decimal;
use num_traits::{ToPrimitive, Zero};

use super::error::BcError;

impl super::BcExecuter {
    pub(super) fn show_limits(&self) -> String {
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

    pub(super) fn format_result(&self, value: Decimal) -> String {
        if self.runtime.obase() == 10 {
            return self.format_result_decimal(&value);
        }
        if let Some(formatted) = self.format_result_obase(&value) {
            formatted
        } else {
            self.format_result_decimal(&value)
        }
    }

    pub(super) fn decimal_from_f64(&self, value: f64, err: &str) -> Result<Decimal, BcError> {
        let decimal = Self::decimal_from_f64_static(value, err)?;
        Ok(self.promote_precision(decimal))
    }

    pub(super) fn decimal_from_f64_static(value: f64, err: &str) -> Result<Decimal, BcError> {
        value
            .to_string()
            .parse::<Decimal>()
            .map_err(|_| BcError::Error(err.to_string()))
    }

    pub(super) fn promote_precision(&self, value: Decimal) -> Decimal {
        const PRECISION_PADDING: usize = 4;
        let digits = value.repr().digits().max(1);
        let target = digits
            .saturating_mul(2)
            .saturating_add(self.runtime.scale() as usize)
            .saturating_add(PRECISION_PADDING)
            .max(1);
        match value.with_precision(target) {
            Approximation::Exact(v) | Approximation::Inexact(v, _) => v,
        }
    }

    pub(crate) fn truncate_decimal_to_scale(value: &Decimal, scale: u32) -> Decimal {
        if scale == 0 {
            return value.trunc();
        }

        let factor = Decimal::from(10).powi(scale.into());
        let truncated = (value.clone() * &factor).trunc();
        truncated / factor
    }

    pub(super) fn decimal_to_plain_string(value: &Decimal) -> String {
        let repr = value.repr();
        if repr.significand().is_zero() {
            return "0".to_string();
        }

        let mut digits = repr.significand().unsigned_abs().to_string();
        let exponent = repr.exponent();

        if exponent >= 0 {
            digits.extend(iter::repeat_n('0', exponent as usize));
        } else {
            let shift = (-exponent) as usize;
            if digits.len() <= shift {
                let mut buffer = String::from("0.");
                buffer.extend(iter::repeat_n('0', shift - digits.len()));
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

    pub(crate) fn format_result_decimal(&self, value: &Decimal) -> String {
        let scale = self.runtime.scale();
        let truncated = Self::truncate_decimal_to_scale(value, scale);
        let mut formatted = Self::decimal_to_plain_string(&truncated);

        if let Some(point_index) = formatted.find('.') {
            if scale == 0 {
                formatted.truncate(point_index);
            } else {
                let frac_len = formatted.len() - point_index - 1;
                if frac_len < scale as usize {
                    formatted.extend(iter::repeat_n('0', scale as usize - frac_len));
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
        let base = self.runtime.obase() as i32;
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
        let scale = self.runtime.scale();
        if !fraction.is_zero() && scale > 0 {
            result.push('.');
            let base_decimal = Decimal::from(base as i64);
            let mut digits_written = 0;
            while digits_written < scale {
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
}
