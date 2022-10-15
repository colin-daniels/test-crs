use crate::enum_token;
use std::fmt::{Display, Formatter};
use thiserror::Error;

enum_token! {
    pub enum OperatorType {
        /// Returns true if the parameter string is found anywhere in the input. Macro expansion
        /// is performed on the parameter string before comparison.
        Contains             = "contains",
        /// Returns true if SQL injection payload is found. This operator uses LibInjection to
        /// detect SQLi attacks.
        ///
        /// Note : This operator supports the "capture" action.
        DetectSQLi           = "detectSQLi",
        /// Returns true if XSS injection is found. This operator uses LibInjection to detect XSS
        /// attacks.
        ///
        /// Note : This operator supports the "capture" action.
        DetectXSS            = "detectXSS",
        /// Returns true if the parameter string is found at the end of the input. Macro expansion
        /// is performed on the parameter string before comparison.
        EndsWith             = "endsWith",
        /// Performs numerical comparison and returns true if the input value is equal to the
        /// provided parameter. Macro expansion is performed on the parameter string before
        /// comparison.
        ///
        /// Note: If a value is provided that cannot be converted to an integer (i.e a string) this
        /// operator will treat that value as 0.
        Eq                   = "eq",
        /// Performs numerical comparison and returns true if the input value is greater than
        /// or equal to the provided parameter. Macro expansion is performed on the parameter
        /// string before comparison.
        ///
        /// Note: If a value is provided that cannot be converted to an integer (i.e a string) this
        /// operator will treat that value as 0.
        Ge                   = "ge",
        /// Performs numerical comparison and returns true if the input value is greater than the
        /// operator parameter. Macro expansion is performed on the parameter string before
        /// comparison.
        ///
        /// Note: If a value is provided that cannot be converted to an integer (i.e a string) this
        /// operator will treat that value as 0.
        Gt                   = "gt",
        /// Performs numerical comparison and returns true if the input value is less than the
        /// operator parameter. Macro expansion is performed on the parameter string before
        /// comparison.
        ///
        /// Note: If a value is provided that cannot be converted to an integer (i.e a string) this
        /// operator will treat that value as 0.
        Lt                   = "lt",
        /// Performs a geolocation lookup using the IP address in input against the geolocation
        /// database previously configured using SecGeoLookupDb. If the lookup is successful, the
        /// obtained information is captured in the GEO collection.
        ///
        /// Note: The geoLookup operator matches on success.
        GeoLookup            = "geoLookup",
        /// Performs a fast ipv4 or ipv6 match of REMOTE_ADDR variable data.
        IpMatch              = "ipMatch",
        /// Performs a fast ipv4 or ipv6 match of REMOTE_ADDR variable, loading data from a file.
        IpMatchFromFile      = "ipMatchFromFile",
        /// Performs a case-insensitive match of the provided phrases against the desired input
        /// value. The operator uses a set-based matching algorithm (Aho-Corasick), which means
        /// that it will match any number of keywords in parallel. When matching of a large number
        /// of keywords is needed, this operator performs much better than a regular expression.
        PatternMatch         = "pm",
        /// This operator is the same as @pm, except that it takes a list of files as arguments.
        /// It will match any one of the phrases listed in the file(s) anywhere in the target value.
        ///
        /// Files must contain exactly one phrase per line. End of line markers (both LF and CRLF)
        /// will be stripped from each phrase and any whitespace trimmed from both the beginning and
        /// the end. Empty lines and comment lines (those beginning with the # character) will be
        /// ignored.
        PatternMatchFromFile = "pmFromFile",
        /// Looks up the input value in the RBL (real-time block list) given as parameter.
        /// The parameter can be an IPv4 address or a hostname.
        RealtimeBlackhole    = "rbl",
        /// Performs a regular expression match of the pattern provided as parameter.
        /// Regular expressions are handled by the PCRE library http://www.pcre.org.
        ///
        /// ModSecurity compiles its regular expressions with the following settings:
        /// - The entire input is treated as a single line, even when there are newline characters
        ///   present.
        /// - All matches are case-sensitive. If you wish to perform case-insensitive matching, you
        ///   can either use the lowercase transformation function or force case-insensitive
        ///   matching by prefixing the regular expression pattern with the (?i) modifier (a PCRE
        ///   feature; you will find many similar features in the PCRE documentation).
        /// - The PCRE_DOTALL and PCRE_DOLLAR_ENDONLY flags are set during compilation, meaning
        ///   that a single dot will match any character, including the newlines, and a $ end anchor
        ///   will not match a trailing newline character.
        ///
        /// Note: This operator supports the "capture" action.
        Regex                = "rx",
        /// Performs a string comparison and returns true if the parameter string is identical
        /// to the input string. Macro expansion is performed on the parameter string before
        /// comparison.
        StringEquals         = "streq",
        /// Validates that the byte values used in input fall into the range specified by the
        /// operator parameter. This operator matches on an input value that contains bytes that
        /// are not in the specified range.
        ValidateByteRange    = "validateByteRange",
        /// Validates the URL-encoded characters in the provided input string.
        ///
        /// ModSecurity will automatically decode the URL-encoded characters in request parameters,
        /// which means that there is little sense in applying the @validateUrlEncoding operator to
        /// them -- that is, unless you know that some of the request parameters were URL-encoded
        /// more than once.
        ValidateUrlEncoding  = "validateUrlEncoding",
        /// Check whether the input is a valid UTF-8 string.
        ValidateUtf8Encoding = "validateUtf8Encoding",
        /// Returns true if the input value (the needle) is found anywhere within the @within
        /// parameter (the haystack). Macro expansion is performed on the parameter string before
        /// comparison.
        Within               = "within",
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Operator {
    pub op: OperatorType,
    pub arg: Option<String>,
}

#[derive(Error, Debug)]
pub enum OperatorParseError {
    #[error("unknown operator {0}")]
    UnknownOperator(String),
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let op = self.op.name();
        match &self.arg {
            Some(arg) => write!(f, "@{} {}", op, arg),
            None => write!(f, "@{}", op),
        }
    }
}

pub fn parse_operator(op: &str, argument: Option<String>) -> Result<Operator, OperatorParseError> {
    use OperatorParseError::*;
    match OperatorType::from_name(&op) {
        Some(op) => Ok(Operator { op, arg: argument }),
        None => Err(UnknownOperator(op.into()))?,
    }
}
