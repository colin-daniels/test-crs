use crate::engine::SourceType;
use http::Request;
use mime::Mime;
use std::str::Utf8Error;

pub mod json;
pub mod www_form_urlencoded;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to parse mime type from Content-Type header, {0}")]
    MimeTypeParseError(#[from] mime::FromStrError),
    #[error("Content-Type header contains invalid UTF-8, {0}")]
    InvalidUtf8Error(#[from] Utf8Error),
}

pub fn get_content_type<T>(request: &Request<T>) -> Option<Result<Mime, Error>> {
    // https://datatracker.ietf.org/doc/html/rfc7231#section-3.1.1.1
    // 3.1.1.1.  Media Type
    //
    //    HTTP uses Internet media types [RFC2046] in the Content-Type
    //    (Section 3.1.1.5) and Accept (Section 5.3.2) header fields in order
    //    to provide open and extensible data typing and type negotiation.
    //    Media types define both a data format and various processing models:
    //    how to process that data in accordance with each context in which it
    //    is received.
    //
    //      media-type = type "/" subtype *( OWS ";" OWS parameter )
    //      type       = token
    //      subtype    = token
    //
    //    The type/subtype MAY be followed by parameters in the form of
    //    name=value pairs.
    //
    //      parameter      = token "=" ( token / quoted-string )
    request
        .headers()
        // FIXME: check for multiple?
        .get(http::header::CONTENT_TYPE)
        .map(|content_type| -> Result<Mime, Error> {
            let content_type = std::str::from_utf8(content_type.as_bytes())?;
            let mime = content_type.parse()?;
            Ok(mime)
        })
}

pub fn parse_www_form_urlencoded(
    data: &[u8],
    source: SourceType,
) -> Result<www_form_urlencoded::Iter, www_form_urlencoded::Error> {
    www_form_urlencoded::Iter::from_bytes(data, source)
}
