use std::fmt::Debug;

use crate::logic::{
    asm::expressions::{
        AsParsedLogicArgument, LogicArgument, LogicBooleanExpression, ParsedLogicArgument,
    },
    logic_script::{
        codegen::{
            context::LogicScriptCodeGenerationContext,
            errors::LogicScriptCodeGenerationError,
            node_label_map::LabeledNode,
            statement_graph::{LogicScriptStatementGraph, LogicScriptStatementGraphNode},
        },
        directives::Directive,
        identifiers::{IdentifierMap, IdentifierMapping},
        operators::{LogicScriptArithmeticOperator, LogicScriptUnaryAssignmentOperator},
        statements::{
            LogicScriptCommandCall, LogicScriptIfStatement, LogicScriptKeyword,
            LogicScriptStatement, LogicScriptStatementBody, StatementWithOrWithoutLocation,
        },
    },
};

fn arg_represents_variable<Arg: AsParsedLogicArgument>(
    argument: &Arg,
    identifier_map: &IdentifierMap,
) -> bool {
    match argument.as_parsed() {
        ParsedLogicArgument::Literal(_) => false,
        ParsedLogicArgument::Identifier(value) => matches!(
            identifier_map.get(&value.name),
            Some(IdentifierMapping::Variable { .. })
        ),
    }
}

#[derive(Debug, Clone)]
pub struct LogicScriptPrimitiveIfStatement<Arg: LogicArgument> {
    pub conditions: LogicBooleanExpression<Arg>,
    pub then_statements: Vec<LogicScriptPrimitiveStatement>,
    pub else_statements: Vec<LogicScriptPrimitiveStatement>,
    pub if_keyword: LogicScriptKeyword,
    pub else_keyword: Option<LogicScriptKeyword>,
}

#[derive(Debug, Clone)]
pub enum LogicScriptPrimitiveStatementBody {
    CommandCall(LogicScriptCommandCall<ParsedLogicArgument>),
    IfStatement(LogicScriptPrimitiveIfStatement<ParsedLogicArgument>),
}

impl LogicScriptPrimitiveStatementBody {
    pub fn get_goto_target_label(&self) -> Option<&String> {
        let LogicScriptPrimitiveStatementBody::CommandCall(statement) = self else {
            return None;
        };

        if statement.command_name != "goto" {
            return None;
        }

        let Some(ParsedLogicArgument::Identifier(target)) = statement
            .argument_list
            .first()
            .map(AsParsedLogicArgument::as_parsed)
        else {
            return None;
        };

        Some(&target.name)
    }
}

#[derive(Debug, Clone)]
pub struct LogicScriptPrimitiveStatement {
    pub body: LogicScriptPrimitiveStatementBody,
    pub label: Option<String>,
}

impl LogicScriptPrimitiveStatement {
    pub fn simplify_statement_graph<Arg: LogicArgument + AsParsedLogicArgument + Clone + Debug>(
        mut statement_graph: LogicScriptStatementGraph<LogicScriptStatement<Arg>>,
    ) -> Result<(Vec<Self>, IdentifierMap, Vec<Directive>), LogicScriptCodeGenerationError> {
        let statements = statement_graph.to_statements()?;
        let directives = statements
            .iter()
            .filter_map(|stmt| match &stmt.body {
                LogicScriptStatementBody::Directive(directive) => Some(directive.directive.clone()),
                _ => None,
            })
            .collect();
        Ok((
            statements
                .into_iter()
                .filter_map(|stmt| {
                    LogicScriptPrimitiveStatement::try_from_statement(&stmt, &statement_graph)
                })
                .collect(),
            statement_graph.identifiers,
            directives,
        ))
    }

    pub fn try_from_statement<Arg: LogicArgument + AsParsedLogicArgument + Clone + Debug>(
        statement: &LogicScriptStatement<ParsedLogicArgument>,
        statement_graph: &LogicScriptStatementGraph<LogicScriptStatement<Arg>>,
    ) -> Option<Self> {
        let primitive_body: LogicScriptPrimitiveStatementBody = match &statement.body {
            LogicScriptStatementBody::UnaryOperation(body) => {
                LogicScriptPrimitiveStatementBody::CommandCall(LogicScriptCommandCall {
                    command_name: match body.operation {
                        LogicScriptUnaryAssignmentOperator::Increment => "increment",
                        LogicScriptUnaryAssignmentOperator::Decrement => "decrement",
                    }
                    .to_string(),
                    argument_list: vec![ParsedLogicArgument::Identifier(body.identifier.clone())],
                })
            }
            LogicScriptStatementBody::ValueAssignment(body) => {
                LogicScriptPrimitiveStatementBody::CommandCall(LogicScriptCommandCall {
                    command_name: if arg_represents_variable(
                        &body.value,
                        &statement_graph.identifiers,
                    ) {
                        "assignv".to_string()
                    } else {
                        "assignn".to_string()
                    },
                    argument_list: vec![
                        ParsedLogicArgument::Identifier(body.assignee.clone()),
                        body.value.as_parsed().clone(),
                    ],
                })
            }
            LogicScriptStatementBody::ArithmeticAssignment(body) => {
                let command_prefix = match body.operator {
                    LogicScriptArithmeticOperator::Add => "add",
                    LogicScriptArithmeticOperator::Subtract => "sub",
                    LogicScriptArithmeticOperator::Multiply => "mul.",
                    LogicScriptArithmeticOperator::Divide => "div.",
                };
                let command_suffix =
                    if arg_represents_variable(&body.value, &statement_graph.identifiers) {
                        "v"
                    } else {
                        "n"
                    };
                LogicScriptPrimitiveStatementBody::CommandCall(LogicScriptCommandCall {
                    command_name: format!("{command_prefix}{command_suffix}"),
                    argument_list: vec![
                        ParsedLogicArgument::Identifier(body.assignee.clone()),
                        body.value.as_parsed().clone(),
                    ],
                })
            }
            LogicScriptStatementBody::LeftIndirectAssignment(body) => {
                LogicScriptPrimitiveStatementBody::CommandCall(LogicScriptCommandCall {
                    command_name: if arg_represents_variable(
                        &body.value,
                        &statement_graph.identifiers,
                    ) {
                        "lindirectv"
                    } else {
                        "lindirectn"
                    }
                    .to_string(),
                    argument_list: vec![
                        ParsedLogicArgument::Identifier(body.assignee_pointer.clone()),
                        body.value.as_parsed().clone(),
                    ],
                })
            }
            LogicScriptStatementBody::RightIndirectAssignment(body) => {
                LogicScriptPrimitiveStatementBody::CommandCall(LogicScriptCommandCall {
                    command_name: "rindirect".to_string(),
                    argument_list: vec![
                        ParsedLogicArgument::Identifier(body.assignee.clone()),
                        ParsedLogicArgument::Identifier(body.value_pointer.clone()),
                    ],
                })
            }
            LogicScriptStatementBody::IfStatement(body) => {
                LogicScriptPrimitiveStatementBody::IfStatement(LogicScriptPrimitiveIfStatement {
                    if_keyword: body.if_keyword.clone(),
                    else_keyword: body.else_keyword.clone(),
                    conditions: body.conditions.to_parsed(),
                    then_statements: body
                        .then_statements
                        .iter()
                        .filter_map(|stmt| {
                            LogicScriptPrimitiveStatement::try_from_statement(
                                stmt.statement(),
                                statement_graph,
                            )
                        })
                        .collect(),
                    else_statements: body
                        .else_statements
                        .iter()
                        .filter_map(|stmt| {
                            LogicScriptPrimitiveStatement::try_from_statement(
                                stmt.statement(),
                                statement_graph,
                            )
                        })
                        .collect(),
                })
            }
            LogicScriptStatementBody::CommandCall(body) => {
                LogicScriptPrimitiveStatementBody::CommandCall(LogicScriptCommandCall {
                    command_name: body.command_name.clone(),
                    argument_list: body
                        .argument_list
                        .iter()
                        .map(|arg| arg.as_parsed().clone())
                        .collect(),
                })
            }
            LogicScriptStatementBody::Directive(_) | LogicScriptStatementBody::Comment(_) => {
                return None;
            }
        };

        Some(LogicScriptPrimitiveStatement {
            body: primitive_body,
            label: statement.label.clone(),
        })
    }

    pub fn to_statement(&self) -> LogicScriptStatement<ParsedLogicArgument> {
        let body: LogicScriptStatementBody<ParsedLogicArgument> = match &self.body {
            LogicScriptPrimitiveStatementBody::CommandCall(body) => {
                LogicScriptStatementBody::CommandCall(body.clone())
            }
            LogicScriptPrimitiveStatementBody::IfStatement(body) => {
                LogicScriptStatementBody::IfStatement(LogicScriptIfStatement {
                    conditions: body.conditions.clone(),
                    then_statements: body
                        .then_statements
                        .iter()
                        .map(|s| StatementWithOrWithoutLocation::WithoutLocation(s.to_statement()))
                        .collect(),
                    else_statements: body
                        .else_statements
                        .iter()
                        .map(|s| StatementWithOrWithoutLocation::WithoutLocation(s.to_statement()))
                        .collect(),
                    if_keyword: body.if_keyword.clone(),
                    else_keyword: body.else_keyword.clone(),
                })
            }
        };

        LogicScriptStatement {
            body,
            label: self.label.clone(),
        }
    }
}

impl LabeledNode for LogicScriptPrimitiveStatement {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn set_label(&mut self, label: Option<&str>) {
        self.label = label.map(|l| l.to_string())
    }
}

impl AsRef<LogicScriptPrimitiveStatement> for LogicScriptPrimitiveStatement {
    fn as_ref(&self) -> &LogicScriptPrimitiveStatement {
        self
    }
}

impl LogicScriptStatementGraphNode for LogicScriptPrimitiveStatement {
    type SubclauseStatement = Self;

    fn get_goto_target_label(&self) -> Option<&str> {
        self.body.get_goto_target_label().map(|l| l.as_str())
    }

    #[cfg(feature = "dot")]
    fn node_attrs(&self, context: &LogicScriptCodeGenerationContext) -> String {
        self.to_statement().node_attrs(context)
    }

    fn if_subclauses(&self) -> Option<(&[Self::SubclauseStatement], &[Self::SubclauseStatement])> {
        match &self.body {
            LogicScriptPrimitiveStatementBody::IfStatement(body) => Some((
                body.then_statements.as_slice(),
                body.else_statements.as_slice(),
            )),
            _ => None,
        }
    }
}
