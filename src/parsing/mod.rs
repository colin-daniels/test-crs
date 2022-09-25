use pest::iterators::Pair;
use pest::Parser;
use std::path::{Path, PathBuf};
use std::{fs, io};

mod util;

#[derive(pest_derive::Parser)]
#[grammar = "parsing/crs.pest"]
struct CRSParser;

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

#[derive(Debug)]
pub struct Input {
    pub modifier: Option<String>,
    pub name: String,
    pub selector: Option<String>,
}

#[derive(Debug)]
pub struct Action {
    pub name: String,
    pub argument: Option<String>,
}

#[derive(Debug)]
pub struct Test {
    pub invert: bool,
    pub operator: String,
    pub argument: Option<String>,
}

pub fn parse_all_conf<P: AsRef<Path>>(dir: P) -> io::Result<Vec<CRSFile>> {
    util::get_rule_configs(dir)?
        .into_iter()
        .map(|path| parse_conf(path))
        .collect()
}

pub fn parse_conf<P: AsRef<Path>>(path: P) -> io::Result<CRSFile> {
    let unparsed_file = fs::read_to_string(path.as_ref())?;

    let file = CRSParser::parse(Rule::crs, &unparsed_file)
        .expect("unsuccessful parse") // unwrap the parse result
        .next()
        .unwrap(); // get and unwrap the `conf` rule

    Ok(CRSFile {
        path: path.as_ref().to_owned(),
        entries: file
            .into_inner()
            .filter_map(|record| match record.as_rule() {
                Rule::sec_marker => Some(CRSEntry::SecMarker(record.into_inner().as_str().into())),
                Rule::sec_action => Some(CRSEntry::SecAction(parse_actions(
                    record.into_inner().next().unwrap(),
                ))),
                Rule::sec_component_signature => Some(CRSEntry::SecComponentSignature(
                    record.into_inner().as_str().into(),
                )),
                Rule::sec_rule => Some(parse_sec_rule(record)),
                Rule::EOI => None,
                _ => unreachable!(),
            })
            .collect(),
    })
}

fn parse_sec_rule(record: Pair<Rule>) -> CRSEntry {
    let mut actions = Default::default();
    let mut inputs = Default::default();
    let mut test = None;
    for part in record.into_inner() {
        match part.as_rule() {
            Rule::inputs => {
                inputs = parse_inputs(part);
            }
            Rule::test => {
                test = Some(parse_test(part));
            }
            Rule::actions => {
                actions = parse_actions(part);
            }
            _ => unreachable!(),
        }
    }

    CRSEntry::SecRule {
        actions,
        inputs,
        test: test.expect("secrules should always have a test defined"),
    }
}

fn parse_actions(action_record: Pair<Rule>) -> Vec<Action> {
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

            Action { name, argument }
        })
        .collect()
}

fn parse_inputs(input_record: Pair<Rule>) -> Vec<Input> {
    input_record
        .into_inner()
        .map(|input| {
            let mut name = Default::default();
            let mut selector = None;
            let mut modifier = None;
            for part in input.into_inner() {
                match part.as_rule() {
                    Rule::input_name => {
                        name = part.as_str().into();
                    }
                    Rule::input_selector => {
                        selector = Some(part.as_str().into());
                    }
                    Rule::input_modifier => {
                        modifier = Some(part.as_str().into());
                    }
                    _ => unreachable!(),
                }
            }

            Input {
                modifier,
                name,
                selector,
            }
        })
        .collect()
}

fn parse_test(test_record: Pair<Rule>) -> Test {
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

    Test {
        invert,
        operator,
        argument,
    }
}
