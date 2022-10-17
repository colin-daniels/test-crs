use crate::engine::{SourceType, Value};
use std::str::{Split, Utf8Error};

// https://www.w3.org/TR/2014/REC-html5-20141028/forms.html#url-encoded-form-data
// Note: This form data set encoding is in many ways an aberrant monstrosity, the result
// of many years of implementation accidents and compromises leading to a set of
// requirements necessary for interoperability, but in no way representing good design
// practices. In particular, readers are cautioned to pay close attention to the twisted
// details involving repeated (and in some cases nested) conversions between character
// encodings and byte sequences.
//
// application/x-www-form-urlencoded
// This is the default content type. Forms submitted with this content type must be encoded as follows:
//
// Control names and values are escaped. Space characters are replaced by `+', and then
// reserved characters are escaped as described in [RFC1738], section 2.2:
// Non-alphanumeric characters are replaced by `%HH', a percent sign and two hexadecimal
// digits representing the ASCII code of the character. Line breaks are represented as
// "CR LF" pairs (i.e., `%0D%0A').
// The control names/values are listed in the order they appear in the document. The
// name is separated from the value by `=' and name/value pairs are separated from each
// other by `&'.
//
// Parameters on the application/x-www-form-urlencoded MIME type are ignored.
// In particular, this MIME type does not support the charset parameter.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("application/x-www-form-urlencoded data contains invalid UTF-8, {0}")]
    InvalidUtf8Error(#[from] Utf8Error),
}

pub struct Iter<'a> {
    inner: Split<'a, char>,
    source: SourceType,
}

impl<'a> Iter<'a> {
    pub fn new(data: &'a str, source: SourceType) -> Self {
        Self {
            inner: data.split('&'),
            source,
        }
    }

    pub fn from_bytes(data: &'a [u8], source: SourceType) -> Result<Self, Error> {
        // HTML5 W3C Recommendation 28 October 2014
        //  https://www.w3.org/TR/2014/REC-html5-20141028/forms.html#url-encoded-form-data
        //
        // This algorithm uses as inputs the payload itself, payload, consisting of a Unicode
        // string using only characters in the range U+0000 to U+007F.
        Ok(Self::new(std::str::from_utf8(data)?, source))
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Value<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // HTML5 W3C Recommendation 28 October 2014
        //  https://www.w3.org/TR/2014/REC-html5-20141028/forms.html#url-encoded-form-data
        let source = self.source;
        self.inner
            .find_map(move |string| match string.split_once('=') {
                // If string contains a "=" (U+003D) character, then let name be the substring
                // of string from the start of string up to but excluding its first "=" (U+003D)
                // character, and let value be the substring from the first character, if any,
                // after the first "=" (U+003D) character up to the end of string. If the first
                // "=" (U+003D) character is the first character, then name will be the empty
                // string. If it is the last character, then value will be the empty string.
                Some((name, value)) => Some(Value::from_str_named(source, name, value)),
                None => {
                    if !string.is_empty() {
                        // Otherwise, string contains no "=" (U+003D) characters. Let name have
                        // the value of string and let value be the empty string.
                        Some(Value::from_str_named(source, string, ""))
                    } else {
                        // Skip totally empty strings
                        None
                    }
                }
            })
    }
}
