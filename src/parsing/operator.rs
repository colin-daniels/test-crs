use thiserror::Error;

#[derive(Debug, Clone)]
pub enum Operator {
    Contains(String),
    EndsWith(String),
    /// String Equality (case sensitive)
    StrEq(String),
    /// Check if the input is within this operator's argument (case-sensitive)
    Within(String),
    /// == (numerical comparison)
    Eq(String),
    /// >= (numerical comparison)
    Ge(String),
    /// > (numerical comparison)
    Gt(String),
    /// < (numerical comparison)
    Lt(String),
    /// Check if the input IP is in the argument list. Argument looks like "127.0.0.1,::1"
    IpMatch(String),
    /// Check if the input IP is in the provided file
    IpMatchFromFile(String),
    /// Pattern match
    Pm(String),
    /// Pattern match from file
    PmFromFile(String),
    /// Real-time blackhole list (RBL) lookup
    Rbl(String),
    /// Regex Match
    Rx(String),
    /// Looks like 9,10,13,32-126,128-255
    ValidateByteRange(String),
    /// Validate URL encoding (e.g., s=a%20b%20c%'/ is invalid)
    ValidateUrlEncoding,
    /// Validate that the input is properly-formed UTF-8.
    ValidateUtf8Encoding,
    /// Geo/Country code lookup
    GeoLookup,
    /// SQLi detection via libinjection
    DetectSQLi,
    /// XSS detection via libinjection
    DetectXSS,
}

#[derive(Error, Debug)]
pub enum OperatorParseError {
    #[error("operator {0} requires an argument")]
    MissingArgument(String),
    #[error("operator {0} does not take an argument (argument: {1})")]
    ExtraArgument(String, String),
    #[error("unknown operator {0}")]
    UnknownOperator(String),
}

pub fn parse_operator(op: &str, argument: Option<String>) -> Result<Operator, OperatorParseError> {
    use OperatorParseError::*;

    let arg = || {
        argument
            .as_ref()
            .map(|arg| arg.clone())
            .ok_or(MissingArgument(op.into()))
    };

    let expect_no_arg = |result| {
        if let Some(arg) = argument.as_ref() {
            Err(ExtraArgument(op.into(), arg.clone()))
        } else {
            Ok(result)
        }
    };

    Ok(match op {
        // String operators
        "contains" => Operator::Contains(arg()?),
        "endsWith" => Operator::EndsWith(arg()?),
        "pm" => Operator::Pm(arg()?),
        "pmFromFile" => Operator::PmFromFile(arg()?),
        "rx" => Operator::Rx(arg()?),
        "streq" => Operator::StrEq(arg()?),
        "validateByteRange" => Operator::ValidateByteRange(arg()?),
        "within" => Operator::Within(arg()?),
        // No-argument string operators
        "detectSQLi" => expect_no_arg(Operator::DetectSQLi)?,
        "detectXSS" => expect_no_arg(Operator::DetectXSS)?,
        "validateUrlEncoding" => expect_no_arg(Operator::ValidateUrlEncoding)?,
        "validateUtf8Encoding" => expect_no_arg(Operator::ValidateUtf8Encoding)?,
        // Numerical operators
        "eq" => Operator::Eq(arg()?),
        "ge" => Operator::Ge(arg()?),
        "gt" => Operator::Gt(arg()?),
        "lt" => Operator::Lt(arg()?),
        // Ip-based operators
        "ipMatch" => Operator::IpMatch(arg()?),
        "ipMatchFromFile" => Operator::IpMatchFromFile(arg()?),
        "geoLookup" => expect_no_arg(Operator::GeoLookup)?,
        "rbl" => Operator::Rbl(arg()?),
        // Unknown
        _ => Err(UnknownOperator(op.into()))?,
    })
}
