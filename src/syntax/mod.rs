use pest::iterators::Pair;
use pest::Parser;
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
    ActionParseError(#[from] action::ActionParseError),
    #[error(transparent)]
    OperatorParseError(#[from] operator::OperatorParseError),
    #[error(transparent)]
    InputParseError(#[from] input::InputParseError),
    #[error(transparent)]
    IoError(#[from] io::Error),
}

#[derive(Debug)]
pub struct CRSFile {
    pub path: PathBuf,
    pub entries: Vec<CRSEntry>,
}

#[derive(Debug)]
pub enum CRSEntry {
    SecMarker(String),
    SecAction(Vec<Action>),
    SecComponentSignature(String),
    SecRule {
        inputs: Vec<Input>,
        test: Test,
        actions: Vec<Action>,
    },
}

#[derive(Debug, Clone)]
pub struct Test {
    pub invert: bool,
    pub operator: Operator,
}

pub fn parse_all_conf<P: AsRef<Path>>(dir: P) -> Result<Vec<CRSFile>, CRSParseError> {
    util::get_rule_configs(dir)?
        .into_iter()
        .map(|path| parse_conf(path))
        .collect()
}

pub fn parse_conf<P: AsRef<Path>>(path: P) -> Result<CRSFile, CRSParseError> {
    let unparsed_file = fs::read_to_string(path.as_ref())?;

    let file = CRSParser::parse(Rule::crs, &unparsed_file)
        .expect("unsuccessful parse") // unwrap the parse result
        .next()
        .unwrap(); // get and unwrap the `conf` rule

    Ok(CRSFile {
        path: path.as_ref().to_owned(),
        entries: file
            .into_inner()
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
            .collect::<Result<_, _>>()?,
    })
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
                        // note: we also trim the single quotes from the argument (if it has any)
                        argument = Some(part.as_str().trim_matches('\'').into());
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
