use bytes::{BufMut, Bytes, BytesMut};
use http::header::{HeaderName, HeaderValue};
use hyper::{Request, Uri};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    YamlError(#[from] serde_yaml::Error),
    #[error(transparent)]
    HttpError(#[from] http::Error),
}

mod defaults {
    #[inline]
    pub fn enabled() -> bool {
        true
    }

    #[inline]
    pub fn addr() -> String {
        "localhost".into()
    }

    #[inline]
    pub fn port() -> u32 {
        80
    }

    #[inline]
    pub fn method() -> String {
        "GET".into()
    }

    #[inline]
    pub fn protocol() -> String {
        "http".into()
    }

    #[inline]
    pub fn uri() -> String {
        "/".into()
    }
}

#[inline]
fn is_false(b: &bool) -> bool {
    *b == false
}

#[inline]
fn is_true(b: &bool) -> bool {
    *b == true
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HttpVersion {
    #[serde(rename = "HTTP/1.0")]
    Http1_0,
    #[serde(rename = "HTTP/1.1")]
    Http1_1,
    #[serde(rename = "HTTP/2", alias = "HTTP/2.0")]
    Http2_0,
    #[serde(other)]
    Unknown,
}

impl Default for HttpVersion {
    fn default() -> Self {
        HttpVersion::Http1_1
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Meta {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub author: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default = "defaults::enabled", skip_serializing_if = "is_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(from = "InputData")]
#[serde(into = "Option<String>")]
pub struct Data(Option<String>);

impl Data {
    pub fn replace_escaped_crlf(&mut self) {
        lazy_static::lazy_static! {
            static ref ESCAPED_CRLF: Regex = Regex::new(r"\\r\\n").unwrap();
        }

        self.0 = match &self.0 {
            Some(text) => Some(ESCAPED_CRLF.replace_all(text.as_str(), "\r\n").into()),
            None => None,
        };
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum InputData {
    None,
    // Note: literals \r and \n will be replaced be replaced with CRLF when stop_magic is on. Note: if
    // urlencoded content-type header is provided and parameters aren't in name=value form, data will
    // be made empty, unless stop_magic is on.
    Text(String),
    Lines(Vec<String>),
}

impl From<InputData> for Data {
    fn from(data: InputData) -> Self {
        // convert request lines into a single string
        Self(match data {
            InputData::None => None,
            InputData::Text(text) => Some(text),
            InputData::Lines(lines) => Some(lines.join("\r\n")),
        })
    }
}

impl From<Data> for Option<String> {
    fn from(data: Data) -> Self {
        data.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// <IP or DNS> def = localhost
    #[serde(default = "defaults::addr")]
    pub dest_addr: String,
    #[serde(default = "defaults::port")]
    pub port: u32,
    #[serde(default = "defaults::method")]
    pub method: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
    /// http or https
    #[serde(default = "defaults::protocol")]
    pub protocol: String,
    #[serde(default = "defaults::uri")]
    pub uri: String,
    #[serde(default = "Default::default")]
    pub version: HttpVersion,
    #[serde(default)]
    pub data: Data,
    /// If there are multiple stages and save cookie is set, it will automatically be provided in
    /// the next stage if the site in question provides the Set-Cookie response header.
    /// Default = false.
    #[serde(default, skip_serializing_if = "is_false")]
    pub save_cookie: bool,
    /// The framework will take care of certain things automatically like setting content-length,
    /// encoding, etc. When stop_magic is on, the framework will not do anything automagically.
    #[serde(default, skip_serializing_if = "is_false")]
    pub stop_magic: bool,
    /// Description: This argument will take a base64 encoded string that will be decoded and sent
    /// through as the request. It will override all other settings
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoded_request: Option<String>,
    /// Description: This argument will take a unencoded string that will be sent through as the
    /// request. It will override all other settings
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_request: Option<String>,
}

impl Input {
    pub fn do_magic(&mut self) {
        // not actually magic, just something we need to do
        self.replace_invalid_query_string_chars();

        // FTW does some things automatically if stop_magic == false, so we do them here.
        if !self.stop_magic {
            // replace "\\r\\n" with actual CRLFs
            self.data.replace_escaped_crlf();

            // default content type to "application/x-www-form-urlencoded" if we have any data
            if let Some(text) = &self.data.0 {
                if !text.is_empty()
                    && !self.headers.contains_key("Content-Type")
                    && !self.headers.contains_key("content-type")
                {
                    self.headers.insert(
                        "Content-Type".into(),
                        "application/x-www-form-urlencoded".into(),
                    );
                }
            }
        }
    }

    fn replace_invalid_query_string_chars(&mut self) {
        // replace any unescaped characters in the query string
        let mut uri_new = BytesMut::with_capacity(self.uri.len() * 3);
        let mut past_query_string = false;

        for &b in self.uri.as_bytes().iter() {
            if !past_query_string {
                past_query_string = b == b'?';
                uri_new.put_u8(b);
            } else {
                match b {
                    b' ' => uri_new.put_slice(b"%20"),
                    b'"' => uri_new.put_slice(b"%22"),
                    b'<' => uri_new.put_slice(b"%3C"),
                    b'>' => uri_new.put_slice(b"%3E"),
                    b'?' => uri_new.put_slice(b"%3F"),
                    _ => uri_new.put_u8(b),
                }
            }
        }

        self.uri = String::from_utf8(uri_new.to_vec()).unwrap();
    }

    pub fn uri(&self) -> Result<Uri, http::Error> {
        Uri::builder()
            .scheme(self.protocol.as_str())
            .authority(format!("localhost:{}", self.port))
            .path_and_query(self.uri.as_str())
            .build()
    }

    pub fn request(&self) -> Result<Request<Option<&str>>, http::Error> {
        let mut builder = Request::builder()
            .method(self.method.as_str())
            .uri(self.uri()?);

        for (header, value) in &self.headers {
            let name = HeaderName::from_bytes(header.as_bytes())?;
            let value: HeaderValue = value.parse()?;
            builder = builder.header(name, value);
        }

        builder.body(self.data.0.as_ref().map(String::as_str))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum OutputStatus {
    Status(u32),
    Any(Vec<u32>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Output {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<OutputStatus>,
    /// Regex
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_contains: Option<String>,
    /// Regex
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_contains: Option<String>,
    /// Regex
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_log_contains: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub expect_error: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage {
    pub input: Input,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<Output>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageWrapper {
    pub stage: Stage,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Test {
    pub test_title: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub desc: String,
    pub stages: Vec<StageWrapper>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct File {
    pub meta: Meta,
    pub tests: Vec<Test>,
    // disallow construction outside of this module?
    #[serde(default, skip)]
    _private: (),
}

impl File {
    pub fn from_str(s: &str) -> Result<Self, Error> {
        serde_yaml::from_str(s)
            .map_err(Into::into)
            .map(|mut file: Self| {
                // FIXME: some sort of initialization magic
                file.inputs_mut().for_each(Input::do_magic);
                file
            })
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Error> {
        Self::from_str(&std::fs::read_to_string(path)?)
    }

    pub fn stages(&self) -> impl Iterator<Item = &Stage> {
        self.tests
            .iter()
            .flat_map(|t| t.stages.iter().map(|s| &s.stage))
    }

    pub fn stages_mut(&mut self) -> impl Iterator<Item = &mut Stage> {
        self.tests
            .iter_mut()
            .flat_map(|t| t.stages.iter_mut().map(|s| &mut s.stage))
    }

    pub fn inputs(&self) -> impl Iterator<Item = &Input> {
        self.stages().map(|s| &s.input)
    }

    pub fn inputs_mut(&mut self) -> impl Iterator<Item = &mut Input> {
        self.stages_mut().map(|s| &mut s.input)
    }
}
