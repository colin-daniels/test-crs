use crate::enum_token;
use std::fmt::{Display, Formatter};
use thiserror::Error;

enum_token! {
    pub enum ActionType {
        /// Marks the transaction for logging in the audit log.
        AuditLog   = "auditlog",
        /// Performs the disruptive action defined by the previous SecDefaultAction.
        Block      = "block",
        /// When used together with the regular expression operator (@rx), the capture action will
        /// create copies of the regular expression captures and place them into the transaction
        /// variable collection.
        Capture    = "capture",
        /// Chains the current rule with the rule that immediately follows it, creating a rule
        /// chain. Chained rules allow for more complex processing logic.
        ///
        /// Rule chains allow you to simulate logical AND. The disruptive actions specified in the
        /// first portion of the chained rule will be triggered only if all of the variable checks
        /// return positive hits. If any one aspect of a chained rule comes back negative, then the
        /// entire rule chain will fail to match. Also note that disruptive actions, execution
        /// phases, metadata actions (id, rev, msg, tag, severity, logdata), skip, and skipAfter
        /// actions can be specified only by the chain starter rule.
        Chain      = "chain",
        /// Changes ModSecurity configuration on transient, per-transaction basis. Any changes made
        /// using this action will affect only the transaction in which the action is executed. The
        /// default configuration, as well as the other transactions running in parallel, will be
        /// unaffected.
        Ctl        = "ctl",
        /// Stops rule processing and intercepts transaction.
        Deny       = "deny",
        /// Initiates an immediate close of the TCP connection by sending a FIN packet.
        Drop       = "drop",
        /// Configures a collection variable to expire after the given time period (in seconds).
        ExpireVar  = "expirevar",
        /// Assigns a unique ID to the rule or chain in which it appears.
        Id         = "id",
        /// Initializes a named persistent collection, either by loading data from storage or by
        /// creating a new collection in memory.
        InitCollection = "initcol",
        /// Indicates that a successful match of the rule needs to be logged.
        Log        = "log",
        /// Logs a data fragment as part of the alert message. Macro expansion is performed, so you
        /// may use variable names such as %{TX.0} or %{MATCHED_VAR}.
        LogData    = "logdata",
        /// Assigns a custom message to the rule or chain in which it appears. The message will be
        /// logged along with every alert.
        Msg        = "msg",
        /// If enabled, ModSecurity will perform multiple operator invocations for every target,
        /// before and after every anti-evasion transformation is performed.
        ///
        /// Normally, variables are inspected only once per rule, and only after all transformation
        /// functions have been completed. With multiMatch, variables are checked against the
        /// operator before and after every transformation function that changes the input.
        MultiMatch = "multiMatch",
        /// Indicates that a successful match of the rule should not be used as criteria to
        /// determine whether the transaction should be logged to the audit log. The noauditlog
        /// action affects only the current rule.
        NoAuditLog = "noauditlog",
        /// Prevents rule matches from appearing in both the error and audit logs.
        /// Implies noauditlog.
        NoLog      = "nolog",
        /// Continues processing with the next rule in spite of a successful match.
        Pass       = "pass",
        /// Places the rule or chain into one of five available processing phases. It can also be
        /// used in SecDefaultAction to establish the rule defaults.
        Phase      = "phase",
        /// Creates, removes, or updates a variable. Variable names are case-insensitive.
        ///
        /// Note: When used in a chain this action will be executed when an individual rule matches
        /// and NOT the entire chain.
        ///
        /// ## Examples
        /// - To create a variable and set its value to 1 (usually used for setting flags),
        ///   use: setvar:TX.score
        /// - To create a variable and initialize it at the same time, use: setvar:TX.score=10
        /// - To remove a variable, prefix the name with an exclamation mark: setvar:!TX.score
        /// - To increase or decrease variable value, use + and - characters in front of a numerical
        ///   value: setvar:TX.score=+5
        Setvar     = "setvar",
        /// Assigns severity to the rule in which it is used. Severity values in ModSecurity follows
        /// the numeric scale of syslog (where 0 is the most severe).
        Severity   = "severity",
        /// Skips one or more rules (or chains) on a successful match, resuming rule execution with
        /// the first rule that follows the rule (or marker created by SecMarker) with the provided
        /// ID.
        SkipAfter  = "skipAfter",
        /// Specifies the response status code to use with actions deny and redirect.
        Status     = "status",
        /// Assigns a tag (category) to a rule or a chain.
        Tag        = "tag",
        /// This action is used to specify the transformation pipeline to use to transform the
        /// value of each variable used in the rule before matching.
        Transform  = "t",
        /// Specifies the rule set version.
        Version    = "ver",
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Action {
    pub action: ActionType,
    pub arg: Option<String>,
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let action = self.action.name();
        match &self.arg {
            Some(arg) => write!(f, "{}:{}", action, arg),
            None => write!(f, "{}", action),
        }
    }
}

#[derive(Error, Debug)]
pub enum ActionParseError {
    #[error("unknown action {0}")]
    UnknownAction(String),
}

pub fn parse_action(action: String, argument: Option<String>) -> Result<Action, ActionParseError> {
    use ActionParseError::*;
    match ActionType::from_name(&action) {
        Some(action) => Ok(Action {
            action,
            arg: argument,
        }),
        None => Err(UnknownAction(action))?,
    }
}
