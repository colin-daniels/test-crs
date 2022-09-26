use super::Rule;
use crate::enum_token;
use pest::iterators::Pair;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Input {
    pub input: InputType,
    pub selector: Selector,
}

enum_token! {
    pub enum InputType {
        ArgsCombinedSize     = "ARGS_COMBINED_SIZE",
        ArgsGetNames         = "ARGS_GET_NAMES",
        ArgsGet              = "ARGS_GET",
        ArgsNames            = "ARGS_NAMES",
        Args                 = "ARGS",
        Duration             = "DURATION",
        FilesCombinedSize    = "FILES_COMBINED_SIZE",
        FilesNames           = "FILES_NAMES",
        Files                = "FILES",
        Geo                  = "GEO",
        Ip                   = "IP",
        MatchedVar           = "MATCHED_VAR",
        MatchedVarsNames     = "MATCHED_VARS_NAMES",
        MatchedVars          = "MATCHED_VARS",
        MultipartPartHeaders = "MULTIPART_PART_HEADERS",
        QueryString          = "QUERY_STRING",
        RemoteAddr           = "REMOTE_ADDR",
        ReqBodyProcessor     = "REQBODY_PROCESSOR",
        RequestBasename      = "REQUEST_BASENAME",
        RequestBody          = "REQUEST_BODY",
        RequestCookiesNames  = "REQUEST_COOKIES_NAMES",
        RequestCookies       = "REQUEST_COOKIES",
        RequestFilename      = "REQUEST_FILENAME",
        RequestHeadersNames  = "REQUEST_HEADERS_NAMES",
        RequestHeaders       = "REQUEST_HEADERS",
        RequestLine          = "REQUEST_LINE",
        RequestMethod        = "REQUEST_METHOD",
        RequestProtocol      = "REQUEST_PROTOCOL",
        RequestUriRaw        = "REQUEST_URI_RAW",
        RequestUri           = "REQUEST_URI",
        ResponseBody         = "RESPONSE_BODY",
        ResponseStatus       = "RESPONSE_STATUS",
        Tx                   = "TX",
        UniqueId             = "UNIQUE_ID",
        Xml                  = "XML",
    }
}

#[derive(Debug, Clone)]
pub enum Selector {
    None,
    Include(String),
    Exclude(String),
    Count(String),
    CountAll,
}

#[derive(Error, Debug)]
pub enum InputParseError {
    #[error("unknown input {0}")]
    UnknownInput(String),
    #[error("invalid input selector {0}")]
    InvalidSelector(String),
    #[error("invalid input modifier {0}")]
    InvalidModifier(String),
}

fn parse_selector(
    modifier: Option<&str>,
    selector: Option<&str>,
    input: &str,
) -> Result<Selector, InputParseError> {
    use InputParseError::*;
    Ok(match (modifier, selector) {
        (Some("!"), Some(s)) => Selector::Exclude(s.into()),
        (Some("!"), None) => Err(InvalidSelector(input.into()))?,
        (Some("&"), Some(s)) => Selector::Count(s.into()),
        (Some("&"), None) => Selector::CountAll,
        (None, Some(s)) => Selector::Include(s.into()),
        (None, None) => Selector::None,
        (Some(_), _) => Err(InvalidModifier(input.into()))?,
    })
}

pub fn parse_input(input_record: Pair<Rule>) -> Result<Input, InputParseError> {
    use InputParseError::*;
    let record = input_record.as_str();

    let mut input_type = None;
    let mut selector = None;
    let mut modifier = None;

    for part in input_record.into_inner() {
        let part_str = part.as_str();
        match part.as_rule() {
            Rule::input_type => {
                input_type = match InputType::from_name(part_str) {
                    Some(input) => Some(input),
                    None => Err(UnknownInput(part_str.into()))?,
                };
            }
            Rule::input_selector => {
                selector = Some(part_str);
            }
            Rule::input_modifier => {
                modifier = Some(part_str);
            }
            _ => unreachable!(),
        }
    }

    Ok(Input {
        input: input_type.expect("input_type should never be None"),
        selector: parse_selector(modifier, selector, record)?,
    })
}
