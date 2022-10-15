use pest::iterators::Pair;
use pest::Parser;
use std::fmt::{Debug, Display, Formatter, Write};
use std::path::{Path, PathBuf};
use std::{fs, io};
use thiserror::Error;

mod action;
mod input;
mod operator;
mod util;

pub use action::Action;
pub use input::{Input, InputType, Selector};
pub use operator::Operator;

#[derive(pest_derive::Parser)]
#[grammar = "syntax/crs.pest"]
struct CRSParser;

#[derive(Error, Debug)]
pub enum CRSParseError {
    #[error(transparent)]
    SyntaxParseError(#[from] pest::error::Error<Rule>),
    #[error(transparent)]
    ActionParseError(#[from] action::ActionParseError),
    #[error(transparent)]
    OperatorParseError(#[from] operator::OperatorParseError),
    #[error(transparent)]
    InputParseError(#[from] input::InputParseError),
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    FmtError(#[from] std::fmt::Error),
    #[error("parsing failure during round-trip validation {0}")]
    RoundTripParseFailed(#[from] Box<CRSParseError>),
    #[error("round-trip validation failed, expected entries: {expected:?}, actual: {actual:?}")]
    RoundTripNotEqual {
        expected: Vec<CRSEntry>,
        actual: Vec<CRSEntry>,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CRSFile {
    pub path: PathBuf,
    pub entries: Vec<CRSEntry>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CRSEntry {
    /// Adds a fixed rule marker that can be used as a target in a skipAfter action.
    /// A SecMarker directive essentially creates a rule that does nothing and whose
    /// only purpose is to carry the given ID.
    SecMarker(String),
    /// Unconditionally processes the action list it receives as the first and only parameter.
    /// The syntax of the parameter is identical to that of the third parameter of SecRule.
    SecAction(Vec<Action>),
    /// Appends component signature to the ModSecurity signature. This directive is used to make
    /// the presence of significant rule sets known. The entire signature will be recorded in the
    /// transaction audit log.
    SecComponentSignature(String),
    /// Creates a rule that will analyze the selected variables using the selected operator.
    SecRule {
        inputs: Vec<Input>,
        test: Test,
        actions: Vec<Action>,
    },
}

fn fmt_iter_join<'a, 'b: 'a, T: Display + 'a>(
    mut iter: impl Iterator<Item = &'a T>,
    f: &mut Formatter<'b>,
    join: char,
) -> std::fmt::Result {
    if let Some(first) = iter.next() {
        first.fmt(f)?;
        iter.map(|v| write!(f, "{}{}", join, v)).collect()
    } else {
        Ok(())
    }
}

fn format_actions(actions: &[Action], f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_char('"')?;
    fmt_iter_join(actions.iter(), f, ',')?;
    f.write_char('"')
}

fn format_inputs(inputs: &[Input], f: &mut Formatter<'_>) -> std::fmt::Result {
    fmt_iter_join(inputs.iter(), f, '|')
}

impl Display for CRSEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CRSEntry::SecMarker(marker) => write!(f, "SecMarker \"{}\"", marker),
            CRSEntry::SecAction(actions) => {
                write!(f, "SecAction ")?;
                format_actions(actions, f)
            }
            CRSEntry::SecComponentSignature(signature) => {
                write!(f, "SecComponentSignature \"{}\"", signature)
            }
            CRSEntry::SecRule {
                inputs,
                test,
                actions,
            } => {
                write!(f, "SecRule ")?;
                format_inputs(inputs, f)?;
                write!(f, " \"{}\"", test)?;
                if !actions.is_empty() {
                    write!(f, " ")?;
                    format_actions(actions, f)
                } else {
                    Ok(())
                }
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Test {
    pub invert: bool,
    pub operator: Operator,
}

impl Display for Test {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.invert {
            write!(f, "!{}", self.operator)
        } else {
            Display::fmt(&self.operator, f)
        }
    }
}

fn parse_entries_impl(input: &str) -> Result<Vec<CRSEntry>, CRSParseError> {
    // get and unwrap the `conf` rule (there's only one, so we can unwrap without worrying)
    let file = CRSParser::parse(Rule::crs, &input)?.next().unwrap();

    file.into_inner()
        .filter(|record| record.as_rule() != Rule::EOI)
        .map(|record| match record.as_rule() {
            Rule::sec_marker => Ok(CRSEntry::SecMarker(record.into_inner().as_str().into())),
            Rule::sec_action => Ok(CRSEntry::SecAction(parse_actions(
                record.into_inner().next().unwrap(),
            )?)),
            Rule::sec_component_signature => Ok(CRSEntry::SecComponentSignature(
                record.into_inner().as_str().into(),
            )),
            Rule::sec_rule => parse_sec_rule(record),
            _ => unreachable!(),
        })
        .collect()
}

fn reserialize(entries: &[CRSEntry]) -> Result<String, std::fmt::Error> {
    let mut reserialized = String::default();
    for entry in entries {
        write!(reserialized, "{}\n", entry)?;
    }
    Ok(reserialized)
}

fn reparse(entries: &[CRSEntry]) -> Result<Vec<CRSEntry>, CRSParseError> {
    let reserialized = reserialize(entries)?;
    match parse_entries_impl(&reserialized) {
        Ok(reparsed) => Ok(reparsed),
        Err(err) => Err(CRSParseError::RoundTripParseFailed(Box::new(err))),
    }
}

pub fn parse_all_conf<P: AsRef<Path>>(dir: P) -> Result<Vec<CRSFile>, CRSParseError> {
    util::get_rule_configs(dir)?
        .into_iter()
        .map(|path| parse_conf(path))
        .collect()
}

pub fn parse_conf<P: AsRef<Path>>(path: P) -> Result<CRSFile, CRSParseError> {
    let file_content = fs::read_to_string(path.as_ref())?;

    Ok(CRSFile {
        path: path.as_ref().to_owned(),
        entries: parse_entries(&file_content)?,
    })
}

pub fn parse_entries(input: &str) -> Result<Vec<CRSEntry>, CRSParseError> {
    let entries = parse_entries_impl(input)?;
    let entries_reparsed = reparse(&entries)?;

    if entries != entries_reparsed {
        Err(CRSParseError::RoundTripNotEqual {
            actual: entries_reparsed,
            expected: entries,
        })
    } else {
        Ok(entries)
    }
}

fn parse_sec_rule(record: Pair<Rule>) -> Result<CRSEntry, CRSParseError> {
    let mut actions = Default::default();
    let mut inputs = Default::default();
    let mut test = None;
    for part in record.into_inner() {
        match part.as_rule() {
            Rule::inputs => {
                inputs = parse_inputs(part)?;
            }
            Rule::test => {
                test = Some(parse_test(part)?);
            }
            Rule::actions => {
                actions = parse_actions(part)?;
            }
            _ => unreachable!(),
        }
    }

    Ok(CRSEntry::SecRule {
        actions,
        inputs,
        test: test.expect("secrules should always have a test defined"),
    })
}

fn parse_actions(action_record: Pair<Rule>) -> Result<Vec<Action>, action::ActionParseError> {
    action_record
        .into_inner()
        .map(|action| {
            let mut name = Default::default();
            let mut argument = None;

            for part in action.into_inner() {
                match part.as_rule() {
                    Rule::action_name => {
                        name = part.as_str().into();
                    }
                    Rule::action_argument => {
                        argument = Some(part.as_str().into());
                    }
                    _ => unreachable!(),
                }
            }

            action::parse_action(name, argument)
        })
        .collect()
}

fn parse_inputs(input_record: Pair<Rule>) -> Result<Vec<Input>, input::InputParseError> {
    input_record.into_inner().map(input::parse_input).collect()
}

fn parse_test(test_record: Pair<Rule>) -> Result<Test, operator::OperatorParseError> {
    let mut invert = false;
    let mut operator = Default::default();
    let mut argument = None;
    for part in test_record.into_inner() {
        match part.as_rule() {
            Rule::test_modifier => {
                assert_eq!(part.as_str(), "!");
                invert = true;
            }
            Rule::test_operator => {
                operator = part.as_str().into();
            }
            Rule::test_argument => {
                argument = Some(part.as_str().into());
            }
            _ => unreachable!(),
        }
    }

    Ok(Test {
        invert,
        operator: operator::parse_operator(operator, argument)?,
    })
}
