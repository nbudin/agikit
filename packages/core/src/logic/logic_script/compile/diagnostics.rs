use std::fmt::Display;

use strum_macros::Display;

use crate::{
    agi_version::AGIVersion,
    logic::{
        asm::expressions::LogicArgument,
        commands::AGICommand,
        logic_script::statements::{LogicScriptStatementBody, StatementWithOrWithoutLocation},
    },
};

#[derive(Debug, Clone)]
pub enum LogicScriptDiagnosticType {
    UnknownCommandName,
    WrongNumberOfArguments,
}

#[derive(Debug, Clone, PartialEq, Display)]
pub enum LogicScriptDiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct LogicScriptDiagnostic<Arg: LogicArgument> {
    pub severity: LogicScriptDiagnosticSeverity,
    pub diagnostic_type: LogicScriptDiagnosticType,
    pub statement: StatementWithOrWithoutLocation<Arg>,
    pub message: String,
}

impl<Arg: LogicArgument> Display for LogicScriptDiagnostic<Arg> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "[{}] {}: {}",
            self.severity,
            self.statement
                .location()
                .map(|location| location.start.to_string())
                .unwrap_or_else(|| "unknown location".to_string()),
            self.message,
        ))
    }
}

impl<Arg: LogicArgument + Clone> LogicScriptDiagnostic<Arg> {
    pub fn for_statement<'a>(
        statement: &'a StatementWithOrWithoutLocation<Arg>,
        agi_version: &'a AGIVersion,
    ) -> Box<dyn Iterator<Item = LogicScriptDiagnostic<Arg>> + 'a> {
        match &statement.statement().body {
            LogicScriptStatementBody::CommandCall(body) => {
                let command = AGICommand::by_name(&body.command_name, agi_version);
                match command {
                    Some(command) => {
                        if body.argument_list.len() != command.arg_types.len() {
                            Box::new(std::iter::once(LogicScriptDiagnostic {
                                severity: LogicScriptDiagnosticSeverity::Error,
                                diagnostic_type: LogicScriptDiagnosticType::WrongNumberOfArguments,
                                statement: statement.clone(),
                                message: format!(
                                    "Wrong number of arguments: expected {}, got {}",
                                    command.arg_types.len(),
                                    body.argument_list.len()
                                ),
                            }))
                        } else {
                            Box::new(std::iter::empty())
                                as Box<dyn Iterator<Item = LogicScriptDiagnostic<Arg>>>
                        }
                    }
                    None => {
                        if body.command_name != "goto" {
                            Box::new(std::iter::once(LogicScriptDiagnostic {
                                severity: LogicScriptDiagnosticSeverity::Error,
                                diagnostic_type: LogicScriptDiagnosticType::UnknownCommandName,
                                statement: statement.clone(),
                                message: format!("Unknown command name: {}", body.command_name),
                            }))
                                as Box<dyn Iterator<Item = LogicScriptDiagnostic<Arg>>>
                        } else {
                            Box::new(std::iter::empty())
                                as Box<dyn Iterator<Item = LogicScriptDiagnostic<Arg>>>
                        }
                    }
                }
            }
            LogicScriptStatementBody::IfStatement(body) => Box::new(
                body.then_statements
                    .iter()
                    .chain(body.else_statements.iter())
                    .flat_map(|stmt| LogicScriptDiagnostic::for_statement(stmt, agi_version)),
            ),
            _ => Box::new(std::iter::empty()),
        }
    }
}
