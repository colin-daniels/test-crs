// use crate::operators::{Action, Operator};
use crate::parsing::CRSEntry;

pub mod expr;
pub mod parsing;

fn main() -> Result<(), parsing::CRSParseError> {
    let config_files = parsing::parse_all_conf("coreruleset/rules")?;

    for conf in config_files {
        println!("File: {}", conf.path.display());
        for entry in conf.entries {
            match entry {
                CRSEntry::SecRule {
                    actions,
                    inputs,
                    test,
                } => {
                    println!("{:?}", test);
                    for action in actions {
                        println!("{:?}", action);
                    }
                    for input in inputs {
                        println!("{:?}", input);
                    }
                }
                CRSEntry::SecAction(actions) => {
                    for action in actions {
                        println!("{:?}", action);
                    }
                }
                _ => (),
            }
        }
    }

    Ok(())
}
