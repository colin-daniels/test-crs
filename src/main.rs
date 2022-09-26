// use crate::operators::{Action, Operator};
use crate::parsing::CRSEntry;

pub mod expr;
pub mod operators;
pub mod parsing;

fn main() -> Result<(), parsing::CRSParseError> {
    let config_files = parsing::parse_all_conf("coreruleset/rules")?;

    // let mut unique_actions = HashSet::new();
    // let mut unique_inputs = HashSet::new();
    // let mut unique_operators = HashSet::new();

    // let mut ops: Vec<Operator> = vec![];
    // let mut acts: Vec<Action> = vec![];
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
                    // match test.clone().try_into() {
                    //     Ok(op) => ops.push(op),
                    //     Err(e) => println!("bad op {:?}\n{}", test, e),
                    // };

                    // unique_operators.insert(test.operator);
                    for action in actions {
                        println!("{:?}", action);
                        // unique_actions.insert(action.name);
                        // match action.clone().try_into() {
                        //     Ok(a) => {
                        //         println!("{:?}", a);
                        //         acts.push(a);
                        //     }
                        //     Err(e) => println!("bad action {:?}\n{}", action, e),
                        // }
                    }
                    for input in inputs {
                        println!("{:?}", input);
                        // unique_inputs.insert(input.input);
                    }
                }
                CRSEntry::SecAction(actions) => {
                    for action in actions {
                        println!("{:?}", action);
                        // unique_actions.insert(action.name);
                        // match action.clone().try_into() {
                        //     Ok(a) => {
                        //         println!("{:?}", a);
                        //         acts.push(a);
                        //     }
                        //     Err(e) => println!("bad action {:?}\n{}", action, e),
                        // }
                    }
                }
                _ => (),
            }
        }
    }

    // println!("{:?}", unique_actions);
    // println!("{:?}", unique_inputs);
    // println!("{:?}", ops);

    Ok(())
}
