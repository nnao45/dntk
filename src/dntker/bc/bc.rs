use std::fmt;

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
    /// Format the result according to the scale setting
    fn format_result(&self, value: f64) -> String {
        let scale = util::DNTK_OPT.scale;

        // Format with the requested scale
        // Note: f64 has limited precision (~15-17 significant digits)
        // Some decimal values cannot be represented exactly in binary floating point
        let formatted = format!("{:.prec$}", value, prec = scale);

        // Remove trailing zeros and decimal point if not needed
        let mut trimmed = formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();

        // bc-compatible formatting: remove leading zero for decimals
        // e.g., "0.333" -> ".333"
        if trimmed.starts_with("0.") {
            trimmed = trimmed.trim_start_matches('0').to_string();
        } else if trimmed.starts_with("-0.") {
            trimmed = format!("-{}", trimmed.trim_start_matches("-0"));
        }

        if trimmed.is_empty() || trimmed == "." || trimmed == "-" {
            "0".to_string()
        } else {
            trimmed
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

        // Preprocess bc-specific function names to standard names
        // bc: s() -> sin(), c() -> cos(), a() -> atan(), l() -> ln(), e() -> exp()
        let processed = self.preprocess_bc_syntax(statement);

        let mut ns = self.create_namespace();

        match fasteval::ez_eval(&processed, &mut ns) {
            Ok(value) => {
                if value.is_nan() || value.is_infinite() {
                    Err(BcError::NoResult)
                } else {
                    Ok(self.format_result(value))
                }
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