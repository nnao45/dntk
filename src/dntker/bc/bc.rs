use std::fmt;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

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

#[derive(Debug, PartialEq)]
pub struct BcExecuter {}

impl Default for BcExecuter {
    fn default() -> Self {
        BcExecuter {}
    }
}

impl BcExecuter {
    /// Format the result according to the scale setting (Decimal version)
    fn format_result(&self, value: Decimal) -> String {
        let scale = util::DNTK_OPT.scale as u32;

        // Round to the specified scale
        let rounded = value.round_dp(scale);

        // Format to string
        let mut formatted = rounded.to_string();

        // Remove trailing zeros
        if formatted.contains('.') {
            formatted = formatted.trim_end_matches('0').trim_end_matches('.').to_string();
        }

        // bc-compatible formatting: remove leading zero for decimals
        // e.g., "0.333" -> ".333"
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

    /// Create a namespace with bc-compatible custom functions
    fn create_namespace(&self) -> fasteval::EmptyNamespace {
        // fasteval has built-in functions, but we need to handle bc-specific ones
        // bc functions: s(x)=sin, c(x)=cos, a(x)=atan, l(x)=ln, e(x)=exp
        // Note: fasteval already has sin, cos, atan, ln, exp, sqrt
        // We just need to make sure the syntax is compatible
        fasteval::EmptyNamespace
    }

    pub fn exec(&self, statement: &str) -> Result<String, BcError> {
        // Handle bc-specific commands
        let trimmed = statement.trim();
        if trimmed == "limits" {
            return Ok(self.show_limits());
        }

        // Use fasteval for parsing and basic evaluation
        // Then convert to Decimal for high-precision operations
        let processed = self.preprocess_bc_syntax(statement);

        // First, try to evaluate with fasteval to get the structure
        let mut ns = self.create_namespace();

        match fasteval::ez_eval(&processed, &mut ns) {
            Ok(f64_value) => {
                if f64_value.is_nan() || f64_value.is_infinite() {
                    return Err(BcError::NoResult);
                }

                // Convert to Decimal for high precision
                let decimal_value = Decimal::from_f64(f64_value)
                    .ok_or_else(|| BcError::Error("Failed to convert to decimal".to_string()))?;

                Ok(self.format_result(decimal_value))
            }
            Err(e) => Err(BcError::Error(format!("{}", e))),
        }
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
            u32::MAX,      // Base max (arbitrary large number)
            65535,         // Dimension max
            i32::MAX,      // Scale max
            i32::MAX,      // String max
            1024,          // Max exponent (f64 exponent range)
            i32::MAX       // Number of variables
        )
    }

    /// Convert bc-specific syntax to standard math syntax
    fn preprocess_bc_syntax(&self, statement: &str) -> String {
        let mut result = statement.to_string();

        // Replace bc-specific function names with standard ones
        // s( -> sin(
        result = result.replace("s(", "sin(");
        // c( -> cos(
        result = result.replace("c(", "cos(");
        // a( -> atan(
        result = result.replace("a(", "atan(");
        // l( -> ln(
        result = result.replace("l(", "ln(");
        // e( -> exp(
        result = result.replace("e(", "exp(");

        // bc uses ^ for exponentiation, fasteval also uses ^, so no change needed

        result
    }
}