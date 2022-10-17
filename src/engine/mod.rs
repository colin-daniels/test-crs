use crate::engine::content_type::parse_www_form_urlencoded;
use bytes::Bytes;
use futures_util::StreamExt;
use http::Request;
use mime::Mime;
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::str::Utf8Error;

pub mod content_type;
pub mod cookies;
pub mod transforms;
pub mod value;

use crate::syntax::{Input, InputType};
use content_type::www_form_urlencoded;
use value::Value;

macro_rules! sources {
    (pub enum $token:ident {
        $(
            $(#[$doc:meta])*
            $variant:ident
        ),*,
    }) => {
        #[derive(Debug, Copy, Clone, Eq, PartialEq)]
        pub enum $token {
            $($(#[$doc])* $variant),*
        }

        impl $token {
            #[inline]
            pub fn variants() -> &'static [Self] {
                &[ $(Self::$variant,)* ]
            }

            #[inline]
            pub fn name(&self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($variant),)*
                }
            }
        }
    };
}

sources! {
    pub enum SourceType {
        Body,
        Cookie,
        CookieName,
        Header,
        HeaderName,
        JsonArg,
        JsonArgName,
        Method,
        PostArg,
        PostArgName,
        Protocol,
        QueryArg,
        QueryArgName,
        UriFull,
        UriPath,
        UriPathAndQuery,
        UriQuery,
        XmlProp,
        XmlPropName,
        XmlText,
    }
}

impl SourceType {
    fn from_modsec_input(input: Input) -> Option<&'static [Self]> {
        match input.input {
            InputType::ArgsGet => Some(&[Self::QueryArg]),
            InputType::ArgsGetNames => Some(&[Self::QueryArgName]),
            InputType::ArgsPost => Some(&[Self::PostArg, Self::JsonArg]),
            InputType::ArgsPostNames => Some(&[Self::PostArgName, Self::JsonArgName]),
            InputType::Args => Some(&[Self::QueryArg, Self::PostArg, Self::JsonArg]),
            InputType::ArgsNames => {
                Some(&[Self::QueryArgName, Self::PostArgName, Self::JsonArgName])
            }
            InputType::QueryString => Some(&[Self::UriQuery]),
            InputType::RequestCookiesNames => Some(&[Self::CookieName]),
            InputType::RequestCookies => Some(&[Self::Cookie]),
            InputType::RequestFilename => Some(&[Self::UriPath]),
            InputType::RequestHeadersNames => Some(&[Self::HeaderName]),
            InputType::RequestHeaders => Some(&[Self::Header]),
            InputType::RequestMethod => Some(&[Self::Method]),
            InputType::RequestProtocol => Some(&[Self::Protocol]),
            InputType::RequestUri => Some(&[Self::UriPathAndQuery]),
            InputType::RequestUriRaw => Some(&[Self::UriFull]),
            InputType::FilesNames => None,
            InputType::Files => None,
            InputType::MultipartPartHeaders => None,
            InputType::RemoteAddr => None,
            InputType::RequestBasename => None,
            InputType::RequestBody => None,
            InputType::RequestLine => None,
            InputType::Xml => None,
            _ => None,
        }
    }
}

impl Display for SourceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

pub trait RequestExt {
    fn cookies(&self) -> Result<cookies::Iter, cookies::Error>;
    fn query_args(&self) -> Option<Result<www_form_urlencoded::Iter, www_form_urlencoded::Error>>;

    fn mime_type(&self) -> Option<Result<Mime, content_type::Error>>;
    fn mime_type_is(&self, mime: &mime::Mime) -> bool {
        self.mime_type().and_then(Result::ok).as_ref() == Some(mime)
    }
}

impl<T> RequestExt for Request<T> {
    fn cookies(&self) -> Result<cookies::Iter, cookies::Error> {
        cookies::Iter::from_request(self)
    }

    fn query_args(&self) -> Option<Result<www_form_urlencoded::Iter, www_form_urlencoded::Error>> {
        let query_bytes = self.uri().query()?.as_bytes();
        Some(parse_www_form_urlencoded(query_bytes, SourceType::QueryArg))
    }

    fn mime_type(&self) -> Option<Result<Mime, content_type::Error>> {
        content_type::get_content_type(self)
    }
}

pub fn get_value_from_source(request: &Request<Vec<u8>>, source: SourceType) -> Vec<Value> {
    use SourceType::*;
    // 2.1. Percent-Encoding: https://datatracker.ietf.org/doc/html/rfc3986#section-2.1
    //    For consistency, percent-encoded octets in the ranges of ALPHA
    //    (%41-%5A and %61-%7A), DIGIT (%30-%39), hyphen (%2D), period (%2E),
    //    underscore (%5F), or tilde (%7E) should not be created by URI
    //    producers and, when found in a URI, should be decoded to their
    //    corresponding unreserved characters by URI normalizers.
    //
    // 2.2. Reserved Characters: https://datatracker.ietf.org/doc/html/rfc3986#section-2.2
    //       reserved    = gen-delims / sub-delims
    //       gen-delims  = ":" / "/" / "?" / "#" / "[" / "]" / "@"
    //       sub-delims  = "!" / "$" / "&" / "'" / "(" / ")"
    //                   / "*" / "+" / "," / ";" / "="
    //
    // https://datatracker.ietf.org/doc/html/rfc3986#section-6.2.2.1
    //    For all URIs, the hexadecimal digits within a percent-encoding
    //    triplet (e.g., "%3a" versus "%3A") are case-insensitive and therefore
    //    should be normalized to use uppercase letters for the digits A-F.
    //
    // https://datatracker.ietf.org/doc/html/rfc3986#section-5.4.2
    // Abnormal Examples
    //
    // https://datatracker.ietf.org/doc/html/rfc3986#section-7.3
    //    Percent-encoded octets must be decoded at some point during the
    //    dereference process.  Applications must split the URI into its
    //    components and subcomponents prior to decoding the octets, as
    //    otherwise the decoded octets might be mistaken for delimiters.
    //
    //    Security checks of the data within a URI should be applied after
    //    decoding the octets.  Note, however, that the "%00" percent-encoding
    //    (NUL) may require special handling and should be rejected if the
    //    application is not expecting to receive raw data within a component.
    match source {
        // HTTP Method
        Method => vec![Value::from_str(Method, request.method().as_str())],

        // Both relative and absolute URIs contain a path component, though it
        // might be the empty string. The path component is **case sensitive**.
        //
        // ```notrust
        // abc://username:password@example.com:123/path/data?key=value&key2=value2#fragid1
        //                                        |--------|
        //                                             |
        //                                           path
        // ```
        UriPath => vec![Value::from_str(UriPath, request.uri().path())],

        // Query String Only
        UriQuery => request
            .uri()
            .query()
            .map(|query| vec![Value::from_str(UriQuery, query)])
            .unwrap_or_default(),

        UriPathAndQuery => request
            .uri()
            .path_and_query()
            .map(|path_and_query| vec![Value::from_str(UriPathAndQuery, path_and_query.as_str())])
            .unwrap_or_default(),

        UriFull => Default::default(),

        // Header Values
        Header => request
            .headers()
            .iter()
            .map(|(name, value)| {
                Value::new_named(Header, name.as_str().as_bytes(), value.as_bytes())
            })
            .collect(),

        // Header Names
        HeaderName => request
            .headers()
            .iter()
            .map(|(name, _)| Value::from_str(HeaderName, name.as_str()))
            .collect(),

        // Cookie Values
        Cookie => request.cookies().map(|c| c.collect()).unwrap_or_default(),

        // Cookie Names
        CookieName => request
            .cookies()
            .map(|iter| iter.filter_map(|v| v.into_name(CookieName)).collect())
            .unwrap_or_default(),

        // Query Arg Values
        QueryArg => {
            // The query component contains non-hierarchical data that, along with
            // data in the path component (Section 3.3), serves to identify a
            // resource within the scope of the URI's scheme and naming authority
            //     (if any).  The query component is indicated by the first question
            // mark ("?") character and terminated by a number sign ("#") character
            // or by the end of the URI.
            request
                .query_args()
                .map(|result| result.unwrap().collect())
                .unwrap_or_default()
        }

        // Query Arg Names
        QueryArgName => request
            .query_args()
            .map(|result| {
                result
                    .unwrap()
                    .filter_map(|v| v.into_name(QueryArgName))
                    .collect()
            })
            .unwrap_or_default(),

        // Post Arg (x-www-form-urlencoded) Values
        PostArg => {
            if request.mime_type_is(&mime::APPLICATION_WWW_FORM_URLENCODED) {
                parse_www_form_urlencoded(request.body(), PostArg)
                    .map(|iter| iter.collect())
                    .unwrap_or_default()
            } else {
                Default::default()
            }
        }

        // Post Arg (x-www-form-urlencoded) Names
        PostArgName => {
            if request.mime_type_is(&mime::APPLICATION_WWW_FORM_URLENCODED) {
                parse_www_form_urlencoded(request.body(), PostArg)
                    .map(|iter| iter.filter_map(|v| v.into_name(PostArgName)).collect())
                    .unwrap_or_default()
            } else {
                Default::default()
            }
        }

        JsonArg => {
            let content_type_is_json: bool = request.mime_type_is(&mime::APPLICATION_JSON);
            let mut deserializer = serde_json::Deserializer::from_slice(request.body());
            Default::default()
        }

        JsonArgName => {
            let content_type_is_json: bool = request.mime_type_is(&mime::APPLICATION_JSON);
            Default::default()
        }

        XmlProp => {
            let content_type_is_xml: bool = request.mime_type_is(&mime::TEXT_XML);
            Default::default()
        }

        XmlPropName => {
            let content_type_is_xml: bool = request.mime_type_is(&mime::TEXT_XML);
            Default::default()
        }

        XmlText => {
            let content_type_is_xml: bool = request.mime_type_is(&mime::TEXT_XML);
            Default::default()
        }

        Protocol => Default::default(),

        Body => vec![Value::new(Body, request.body())],
    }
}
