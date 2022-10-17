use super::{SourceType, Value};
use http::Request;
use std::iter::FilterMap;
use std::str::{Split, Utf8Error};

pub struct Iter<'a> {
    inner: Split<'a, &'static str>,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("invalid utf-8 in cookie header value, {0}")]
    InvalidCookieUTF8Error(#[from] Utf8Error),
    #[error("multiple cookie headers are invalid, see RFC6265")]
    MultipleCookieError,
    #[error("empty cookie pair")]
    EmptyCookieError,
    #[error("invalid cookie pair string, missing '='")]
    InvalidCookiePairError,
}

fn parse_cookie_pair(cookie_pair: &str) -> Option<Value> {
    if cookie_pair.is_empty() {
        // ignore? (it's possible that this is a trailing '; ')
        return None;
    }

    // RFC: https://httpwg.org/specs/rfc6265.html#cookie
    //
    // cookie-pair = cookie-name "=" cookie-value
    //
    // cookie-name = 1*<any CHAR except CTLs or separators>
    // CHAR = <any US-ASCII character (octets 0 - 127)>
    // CTL = <any US-ASCII control character (octets 0 - 31) and DEL (127)>
    // separators = "(" | ")" | "<" | ">" | "@"
    //            | "," | ";" | ":" | "\" | <">
    //            | "/" | "[" | "]" | "?" | "="
    //            | "{" | "}" | SP | HT
    //
    // cookie-value = *cookie-octet | ( DQUOTE *cookie-octet DQUOTE )
    // cookie-octet = %x21 / %x23-2B / %x2D-3A / %x3C-5B / %x5D-7E
    //              ; US-ASCII characters excluding CTLs,
    //              ; whitespace DQUOTE, comma, semicolon,
    //              ; and backslash
    // DQUOTE =  %x22
    //
    match cookie_pair.split_once('=') {
        None => {
            // FIXME: invalid cookie-pair?
            None
        }
        Some((cookie_name, cookie_value)) => {
            // trim off any double quotes from the value
            let cookie_value = cookie_value.trim_matches('"');
            Some(Value::from_str_named(
                SourceType::Cookie,
                cookie_name,
                cookie_value,
            ))
        }
    }
}

impl<'a> Iter<'a> {
    pub fn new(cookie_string: &'a str) -> Self {
        Self {
            inner: cookie_string.split("; "),
        }
    }

    pub fn from_request<T>(request: &'a Request<T>) -> Result<Self, Error> {
        // Ref: https://httpwg.org/specs/rfc6265.html#cookie
        let cookie_string = request
            .headers()
            // When the user agent generates an HTTP request, the user agent MUST NOT attach
            // more than one Cookie header field.
            //
            // Output the cookie's name, the %x3D ("=") character, and the cookie's value.
            // If there is an unprocessed cookie in the cookie-list, output the characters %x3B
            // and %x20 ("; ").
            .get(http::header::COOKIE)
            .map(|cookie_string| {
                // According to RFC6265, valid cookie headers should also be valid UTF-8, since
                // octets greater than 127 are not allowed as part of a cookie-header.
                std::str::from_utf8(cookie_string.as_bytes())
            })
            .unwrap_or(Ok(""))?;

        Ok(Self::new(cookie_string))
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Value<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.find_map(parse_cookie_pair)
    }
}
