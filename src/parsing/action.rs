use thiserror::Error;

#[derive(Debug, Clone)]
pub enum Action {
    AuditLog,
    Block,
    Capture,
    Chain,
    Ctl(String),
    Deny,
    Drop,
    ExpireVar(String),
    Id(String),
    InitCol(String),
    Log,
    LogData(String),
    Msg(String),
    MultiMatch,
    NoAuditLog,
    NoLog,
    Pass,
    Phase(String),
    Setvar(String),
    Severity(String),
    SkipAfter(String),
    Status(String),
    Transform(String),
    Tag(String),
    Version(String),
}

#[derive(Error, Debug)]
pub enum ActionParseError {
    #[error("action {0} requires an argument")]
    MissingArgument(String),
    #[error("action {0} does not take an argument (argument: {1})")]
    ExtraArgument(String, String),
    #[error("unknown action {0}")]
    UnknownAction(String),
}

pub fn parse_action(action: String, argument: Option<String>) -> Result<Action, ActionParseError> {
    use ActionParseError::*;

    let arg = || {
        argument
            .as_ref()
            .map(|arg| arg.clone())
            .ok_or(MissingArgument(action.clone()))
    };

    let expect_no_arg = |result: Action| {
        if let Some(arg) = argument.as_ref() {
            Err(ExtraArgument(action.clone(), arg.clone()))
        } else {
            Ok(result)
        }
    };

    Ok(match action.as_str() {
        // no-argument actions
        "auditlog" => expect_no_arg(Action::AuditLog)?,
        "block" => expect_no_arg(Action::Block)?,
        "capture" => expect_no_arg(Action::Capture)?,
        "chain" => expect_no_arg(Action::Chain)?,
        "deny" => expect_no_arg(Action::Deny)?,
        "drop" => expect_no_arg(Action::Drop)?,
        "log" => expect_no_arg(Action::Log)?,
        "multiMatch" => expect_no_arg(Action::MultiMatch)?,
        "noauditlog" => expect_no_arg(Action::NoAuditLog)?,
        "nolog" => expect_no_arg(Action::NoLog)?,
        "pass" => expect_no_arg(Action::Pass)?,
        // actions with arguments
        "ctl" => Action::Ctl(arg()?),
        "expirevar" => Action::ExpireVar(arg()?),
        "id" => Action::Id(arg()?),
        "initcol" => Action::InitCol(arg()?),
        "logdata" => Action::LogData(arg()?),
        "msg" => Action::Msg(arg()?),
        "phase" => Action::Phase(arg()?),
        "setvar" => Action::Setvar(arg()?),
        "severity" => Action::Severity(arg()?),
        "skipAfter" => Action::SkipAfter(arg()?),
        "status" => Action::Status(arg()?),
        "t" => Action::Transform(arg()?),
        "tag" => Action::Tag(arg()?),
        "ver" => Action::Version(arg()?),
        // unknown action
        _ => Err(UnknownAction(action))?,
    })
}
