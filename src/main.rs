use crate::parsing::CRSEntry;
use std::collections::HashSet;
use std::io;

pub mod parsing;

fn main() -> io::Result<()> {
    let config_files = parsing::parse_all_conf("coreruleset/rules")?;

    let mut unique_actions = HashSet::new();
    let mut unique_inputs = HashSet::new();
    let mut unique_operators = HashSet::new();
    for conf in config_files {
        println!("File: {}", conf.path.display());
        for entry in conf.entries {
            // println!("{:?}", entry);
            match entry {
                CRSEntry::SecRule {
                    actions,
                    inputs,
                    test,
                } => {
                    println!("{:?}", test);
                    unique_operators.insert(test.operator);
                    for action in actions {
                        unique_actions.insert(action.name);
                    }
                    for input in inputs {
                        unique_inputs.insert(input.modifier.unwrap_or_default());
                    }
                }
                CRSEntry::SecAction(actions) => {
                    for action in actions {
                        unique_actions.insert(action.name);
                    }
                }
                _ => (),
            }
        }
    }

    println!("{:?}", unique_actions);
    println!("{:?}", unique_inputs);
    println!("{:?}", unique_operators);

    Ok(())
}
