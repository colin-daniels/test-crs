// use crate::parsing;
// use std::fmt::{Debug, Formatter};
// use std::iter::Peekable;
// use std::net::{AddrParseError, IpAddr};
// use std::num::ParseIntError;
// use std::ops::RangeInclusive;
// use std::str::Chars;
//
// use hyperscan::prelude::*;
// use lazy_static::lazy_static;
//
// use crate::operators::OperatorParseError::{ExtraArgumentError, MissingArgumentError};
// use regex::Regex;
// use thiserror::Error;
//
// pub struct RegexWrapper(BlockDatabase);
//
// impl Debug for RegexWrapper {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         f.write_str("BlockDatabase")
//     }
// }
//
// #[derive(Debug)]
// pub enum Operator {
//     Contains(String),
//     EndsWith(String),
//     /// String Equality (case sensitive)
//     StrEq(String),
//     /// Check if the input is within this operator's argument (case-sensitive)
//     Within(String),
//     /// == (numerical comparison)
//     Eq(String),
//     /// >= (numerical comparison)
//     Ge(String),
//     /// > (numerical comparison)
//     Gt(String),
//     /// < (numerical comparison)
//     Lt(String),
//     /// Check if the input IP is in the argument list. Argument looks like "127.0.0.1,::1"
//     IpMatch(Vec<IpAddr>),
//     /// Check if the input IP is in the provided file
//     IpMatchFromFile(String),
//     /// Pattern match
//     Pm(String),
//     /// Pattern match from file
//     PmFromFile(String),
//     /// Real-time blackhole list (RBL) lookup
//     Rbl(String),
//     /// Regex Match
//     Rx(String),
//     /// Looks like 9,10,13,32-126,128-255
//     ValidateByteRange(Vec<RangeInclusive<u8>>),
//     /// Validate URL encoding (e.g., s=a%20b%20c%'/ is invalid)
//     ValidateUrlEncoding,
//     /// Validate that the input is properly-formed UTF-8.
//     ValidateUtf8Encoding,
//     /// Geo/Country code lookup
//     GeoLookup,
//     /// SQLi detection via libinjection
//     DetectSQLi,
//     /// XSS detection via libinjection
//     DetectXSS,
// }
//
// #[derive(Error, Debug)]
// pub enum OperatorParseError {
//     #[error("operator {0} requires an argument")]
//     MissingArgumentError(String),
//     #[error("operator {0} does not take an argument (argument: {1})")]
//     ExtraArgumentError(String, String),
//     #[error("invalid byte range {0}")]
//     InvalidByteRangeError(String),
//     #[error(transparent)]
//     AddrParseError(#[from] AddrParseError),
//     #[error(transparent)]
//     ParseIntError(#[from] ParseIntError),
//     #[error(transparent)]
//     RegexError(#[from] regex::Error),
// }
//
// fn parse_byte_range(range: &str) -> Result<RangeInclusive<u8>, OperatorParseError> {
//     let mut iter = range.split("-");
//     let start: u8 = iter.next().unwrap().parse()?;
//     let end: u8 = if let Some(end) = iter.next() {
//         end.parse()?
//     } else {
//         start
//     };
//
//     Ok(RangeInclusive::new(start, end))
// }
//
// impl TryFrom<parsing::Test> for Operator {
//     type Error = OperatorParseError;
//
//     fn try_from(t: parsing::Test) -> Result<Self, Self::Error> {
//         let op = t.operator.as_str();
//
//         let arg = || {
//             t.argument
//                 .as_ref()
//                 .map(|arg| arg.clone())
//                 .ok_or(MissingArgumentError(op.into()))
//         };
//
//         let expect_no_arg = || {
//             if let Some(arg) = t.argument.as_ref() {
//                 Err(ExtraArgumentError(op.into(), arg.clone()))
//             } else {
//                 Ok(())
//             }
//         };
//
//         Ok(match op {
//             // String operators
//             "contains" => Operator::Contains(arg()?),
//             "endsWith" => Operator::EndsWith(arg()?),
//             "pm" => Operator::Pm(arg()?),
//             "pmFromFile" => Operator::PmFromFile(arg()?),
//             "rx" => Operator::Rx(arg()?),
//             "streq" => Operator::StrEq(arg()?),
//             "validateByteRange" => Operator::ValidateByteRange(
//                 arg()?
//                     .split(",")
//                     .map(parse_byte_range)
//                     .collect::<Result<_, _>>()?,
//             ),
//             "within" => Operator::Within(arg()?),
//             // No-argument string operators
//             "detectSQLi" => {
//                 expect_no_arg()?;
//                 Operator::DetectSQLi
//             }
//             "detectXSS" => {
//                 expect_no_arg()?;
//                 Operator::DetectXSS
//             }
//             "validateUrlEncoding" => {
//                 expect_no_arg()?;
//                 Operator::ValidateUrlEncoding
//             }
//             "validateUtf8Encoding" => {
//                 expect_no_arg()?;
//                 Operator::ValidateUtf8Encoding
//             }
//             // Numerical operators
//             "eq" => Operator::Eq(arg()?),
//             "ge" => Operator::Ge(arg()?),
//             "gt" => Operator::Gt(arg()?),
//             "lt" => Operator::Lt(arg()?),
//             // Ip-based operators
//             "ipMatch" => Operator::IpMatch(
//                 arg()?
//                     .split(",")
//                     .map(|ip| ip.parse())
//                     .collect::<Result<_, _>>()?,
//             ),
//             "ipMatchFromFile" => Operator::IpMatchFromFile(arg()?),
//             "geoLookup" => {
//                 expect_no_arg()?;
//                 Operator::GeoLookup
//             }
//             "rbl" => Operator::Rbl(arg()?),
//             unknown => {
//                 panic!("unknown operator {}", unknown);
//             }
//         })
//     }
// }
//
// // struct InterpretEscapedString<'a> {
// //     ignore_next: bool,
// //     s: Peekable<Chars<'a>>,
// // }
// //
// // impl<'a> Iterator for InterpretEscapedString<'a> {
// //     type Item = char;
// //
// //     fn next(&mut self) -> Option<Self::Item> {
// //         if self.ignore_next {
// //             self.ignore_next = false;
// //             return self.s.next();
// //         }
// //
// //         let c = self.s.next()?;
// //         let c = match c {
// //             // if the current char is an escape
// //             '\\' => match self.s.peek() {
// //                 None => c,
// //                 // look at the next one to determine if we should escape it or not
// //                 Some(&n) => {
// //                     match n {
// //                         '\\' => {
// //                             // escaped backslash: do nothing, and ignore the next char
// //                             self.ignore_next = true;
// //                             c
// //                         }
// //                         '\"' | '/' | ',' | '!' | '\'' | '%' | '@' | '<' | '>' | '=' => {
// //                             // skip the next one, and return _just_ the escaped char
// //                             self.s.next();
// //                             n
// //                         }
// //                         _ => c,
// //                     }
// //                 }
// //             },
// //             _ => c,
// //         };
// //
// //         Some(c)
// //     }
// // }
// //
// // fn unescape_string(s: &str) -> String {
// //     (InterpretEscapedString {
// //         s: s.chars().peekable(),
// //         ignore_next: false,
// //     })
// //     .collect()
// // }
// //
// // fn process_regex_str(s: &str) -> String {
// //     lazy_static! {
// //         static ref RE_UNESCAPED_CURLY_BRACE: Regex =
// //             Regex::new(r"(?P<l>[^\\%]|^)(?P<r>\{(?:\D|$))").unwrap();
// //         static ref RE_UNESCAPED_BRACKET: Regex = Regex::new(r"(?P<l>[^\\]\[[^]]*[^\\])\[").unwrap();
// //     }
// //
// //     let processed = unescape_string(s);
// //     let processed = RE_UNESCAPED_BRACKET.replace_all(&processed, "$l\\[");
// //     let processed = RE_UNESCAPED_CURLY_BRACE.replace_all(&processed, "$l\\$r");
// //     let processed = RE_UNESCAPED_CURLY_BRACE.replace_all(&processed, "$l\\$r");
// //
// //     processed.to_string()
// //     // match hyperscan::compile(processed.to_string()) {
// //     //     Ok(db) => db,
// //     //     Err(e) => {
// //     //         let re = Regex::new(&processed);
// //     //         if let Err(e) = re {
// //     //             println!("{}", e);
// //     //         }
// //     //         panic!("{}\n{}", e, processed);
// //     //     }
// //     // }
// // }
