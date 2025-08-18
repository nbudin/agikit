use crate::{
    agi_version::AGIVersion,
    logic::{
        asm::expressions::ParsedLogicArgument,
        commands::AGICommand,
        logic_script::{
            locations::WithLocation,
            statements::{LogicScriptStatement, LogicScriptStatementBody},
        },
    },
};

pub enum LogicScriptDiagnosticType {
    UnknownCommandName,
    WrongNumberOfArguments,
}

#[derive(PartialEq)]
pub enum LogicScriptDiagnosticSeverity {
    Warning,
    Error,
}

pub struct LogicScriptDiagnostic {
    pub severity: LogicScriptDiagnosticSeverity,
    pub diagnostic_type: LogicScriptDiagnosticType,
    pub statement: LogicScriptStatement<WithLocation<ParsedLogicArgument>>,
    pub message: String,
}

impl LogicScriptDiagnostic {
    pub fn for_statement<'a>(
        statement: &'a LogicScriptStatement<WithLocation<ParsedLogicArgument>>,
        agi_version: &'a AGIVersion,
    ) -> Box<dyn Iterator<Item = LogicScriptDiagnostic> + 'a> {
        match &statement.body {
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
                                as Box<dyn Iterator<Item = LogicScriptDiagnostic>>
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
                                as Box<dyn Iterator<Item = LogicScriptDiagnostic>>
                        } else {
                            Box::new(std::iter::empty())
                                as Box<dyn Iterator<Item = LogicScriptDiagnostic>>
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
