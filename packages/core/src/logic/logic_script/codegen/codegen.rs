use std::collections::HashMap;

use crate::logic::{
    asm::{codegen::GenerateLogicAsm, expressions::LogicArgument, literals::LogicNumberLiteral},
    logic_script::{
        codegen::{
            context::LogicScriptCodeGenerationContext, errors::LogicScriptCodeGenerationError,
        },
        directives::{Directive, LogicScriptDefineValue},
        literals::{LogicScriptLiteral, LogicScriptLiteralValue, LogicScriptStringLiteral},
        operators::{LogicScriptArithmeticOperator, LogicScriptUnaryAssignmentOperator},
        statements::{
            LogicScriptArithmeticAssignmentStatement, LogicScriptCommandCall, LogicScriptComment,
            LogicScriptLeftIndirectAssignmentStatement,
            LogicScriptRightIndirectAssignmentStatement, LogicScriptStatement,
            LogicScriptStatementBody, LogicScriptUnaryOperationStatement,
            LogicScriptValueAssignmentStatement, StatementWithOrWithoutLocation,
        },
    },
};

pub trait GenerateLogicScript {
    type Options;

    fn generate_logic_script(
        &self,
        context: &LogicScriptCodeGenerationContext<'_>,
        options: Self::Options,
    ) -> Result<String, LogicScriptCodeGenerationError>;
}

impl<T: GenerateLogicAsm> GenerateLogicScript for T {
    type Options = ();

    fn generate_logic_script(
        &self,
        context: &LogicScriptCodeGenerationContext<'_>,
        _options: Self::Options,
    ) -> Result<String, LogicScriptCodeGenerationError> {
        self.generate_asm(&context.asm_context, &HashMap::new())
            .map_err(LogicScriptCodeGenerationError::AsmCodeGenerationError)
    }
}

impl GenerateLogicScript for LogicNumberLiteral {
    type Options = ();

    fn generate_logic_script(
        &self,
        _context: &LogicScriptCodeGenerationContext<'_>,
        _options: Self::Options,
    ) -> Result<String, LogicScriptCodeGenerationError> {
        Ok(format!("{}", self.value))
    }
}

impl GenerateLogicScript for LogicScriptStringLiteral {
    type Options = ();

    fn generate_logic_script(
        &self,
        _context: &LogicScriptCodeGenerationContext<'_>,
        _options: Self::Options,
    ) -> Result<String, LogicScriptCodeGenerationError> {
        Ok(serde_json::to_string(&self.value())?)
    }
}

impl GenerateLogicScript for LogicScriptLiteral {
    type Options = ();

    fn generate_logic_script(
        &self,
        context: &LogicScriptCodeGenerationContext<'_>,
        _options: Self::Options,
    ) -> Result<String, LogicScriptCodeGenerationError> {
        match self.value {
            LogicScriptLiteralValue::Number(ref num) => num.generate_logic_script(context, ()),
            LogicScriptLiteralValue::String(ref string) => {
                string.generate_logic_script(context, ())
            }
        }
    }
}

impl GenerateLogicScript for LogicScriptComment {
    type Options = ();

    fn generate_logic_script(
        &self,
        _context: &LogicScriptCodeGenerationContext<'_>,
        _options: Self::Options,
    ) -> Result<String, LogicScriptCodeGenerationError> {
        Ok(format!("//{}\n", self.comment))
    }
}

impl GenerateLogicScript for LogicScriptDefineValue {
    type Options = ();

    fn generate_logic_script(
        &self,
        context: &LogicScriptCodeGenerationContext<'_>,
        _options: Self::Options,
    ) -> Result<String, LogicScriptCodeGenerationError> {
        match self {
            LogicScriptDefineValue::Literal(literal) => literal.generate_logic_script(context, ()),
            LogicScriptDefineValue::Identifier(identifier) => {
                identifier.generate_logic_script(context, ())
            }
        }
    }
}

impl GenerateLogicScript for Directive {
    type Options = ();

    fn generate_logic_script(
        &self,
        context: &LogicScriptCodeGenerationContext<'_>,
        _options: Self::Options,
    ) -> Result<String, LogicScriptCodeGenerationError> {
        match self {
            Directive::Message { number, message } => Ok(format!(
                "#message {} {}",
                number.value,
                serde_json::to_string(&message.value())?
            )),
            Directive::Include { filename } => Ok(format!(
                "#include {}",
                serde_json::to_string(&filename.value())?
            )),
            Directive::Define { identifier, value } => Ok(format!(
                "#define {} {}",
                identifier.generate_logic_script(context, ())?,
                value.generate_logic_script(context, ())?
            )),
        }
    }
}

impl<Arg: LogicArgument> GenerateLogicScript for Vec<Arg> {
    type Options = ();

    fn generate_logic_script(
        &self,
        context: &LogicScriptCodeGenerationContext<'_>,
        _options: Self::Options,
    ) -> Result<String, LogicScriptCodeGenerationError> {
        self.iter()
            .map(|arg| {
                arg.try_parse(&context.asm_context)
                    .map_err(LogicScriptCodeGenerationError::AsmCodeGenerationError)
                    .and_then(|parsed| parsed.generate_logic_script(context, ()))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|args| args.join(", "))
    }
}

impl<Arg: LogicArgument> GenerateLogicScript for LogicScriptCommandCall<Arg> {
    type Options = ();

    fn generate_logic_script(
        &self,
        context: &LogicScriptCodeGenerationContext<'_>,
        options: Self::Options,
    ) -> Result<String, LogicScriptCodeGenerationError> {
        Ok(format!(
            "{}({});",
            self.command_name,
            self.argument_list.generate_logic_script(context, options)?
        ))
    }
}

impl GenerateLogicScript for LogicScriptUnaryAssignmentOperator {
    type Options = ();

    fn generate_logic_script(
        &self,
        _context: &LogicScriptCodeGenerationContext<'_>,
        _options: Self::Options,
    ) -> Result<String, LogicScriptCodeGenerationError> {
        match self {
            LogicScriptUnaryAssignmentOperator::Increment => Ok("++".to_string()),
            LogicScriptUnaryAssignmentOperator::Decrement => Ok("--".to_string()),
        }
    }
}

impl GenerateLogicScript for LogicScriptUnaryOperationStatement {
    type Options = ();

    fn generate_logic_script(
        &self,
        context: &LogicScriptCodeGenerationContext<'_>,
        options: Self::Options,
    ) -> Result<String, LogicScriptCodeGenerationError> {
        Ok(format!(
            "{}{};",
            self.identifier.generate_logic_script(context, options)?,
            self.operation.generate_logic_script(context, options)?
        ))
    }
}

impl<Arg: LogicArgument + GenerateLogicAsm> GenerateLogicScript
    for LogicScriptValueAssignmentStatement<Arg>
{
    type Options = ();

    fn generate_logic_script(
        &self,
        context: &LogicScriptCodeGenerationContext<'_>,
        options: Self::Options,
    ) -> Result<String, LogicScriptCodeGenerationError> {
        Ok(format!(
            "{} = {};",
            self.assignee.generate_logic_script(context, options)?,
            self.value.generate_logic_script(context, options)?
        ))
    }
}

impl GenerateLogicScript for LogicScriptArithmeticOperator {
    type Options = ();

    fn generate_logic_script(
        &self,
        _context: &LogicScriptCodeGenerationContext<'_>,
        _options: Self::Options,
    ) -> Result<String, LogicScriptCodeGenerationError> {
        match self {
            LogicScriptArithmeticOperator::Add => Ok("+".to_string()),
            LogicScriptArithmeticOperator::Subtract => Ok("-".to_string()),
            LogicScriptArithmeticOperator::Multiply => Ok("*".to_string()),
            LogicScriptArithmeticOperator::Divide => Ok("/".to_string()),
        }
    }
}

impl<Arg: LogicArgument + GenerateLogicAsm> GenerateLogicScript
    for LogicScriptArithmeticAssignmentStatement<Arg>
{
    type Options = ();

    fn generate_logic_script(
        &self,
        context: &LogicScriptCodeGenerationContext<'_>,
        options: Self::Options,
    ) -> Result<String, LogicScriptCodeGenerationError> {
        Ok(format!(
            "{} {}= {};",
            self.assignee.generate_logic_script(context, options)?,
            self.operator.generate_logic_script(context, options)?,
            self.value.generate_logic_script(context, options)?
        ))
    }
}

impl<Arg: LogicArgument + GenerateLogicAsm> GenerateLogicScript
    for LogicScriptLeftIndirectAssignmentStatement<Arg>
{
    type Options = (); // indentation level

    fn generate_logic_script(
        &self,
        context: &LogicScriptCodeGenerationContext<'_>,
        options: Self::Options,
    ) -> Result<String, LogicScriptCodeGenerationError> {
        Ok(format!(
            "*{} = {};",
            self.assignee_pointer
                .generate_logic_script(context, options)?,
            self.value.generate_logic_script(context, options)?
        ))
    }
}

impl GenerateLogicScript for LogicScriptRightIndirectAssignmentStatement {
    type Options = (); // indentation level

    fn generate_logic_script(
        &self,
        context: &LogicScriptCodeGenerationContext<'_>,
        options: Self::Options,
    ) -> Result<String, LogicScriptCodeGenerationError> {
        Ok(format!(
            "{} = *{};",
            self.assignee.generate_logic_script(context, options)?,
            self.value_pointer.generate_logic_script(context, options)?
        ))
    }
}

impl<Arg: LogicArgument + GenerateLogicAsm + Clone> GenerateLogicScript
    for LogicScriptStatementBody<Arg>
{
    type Options = usize; // indentation level

    fn generate_logic_script(
        &self,
        context: &LogicScriptCodeGenerationContext<'_>,
        indent: Self::Options,
    ) -> Result<String, LogicScriptCodeGenerationError> {
        let indent_line = |line: &str| -> String {
            format!("{}{}", " ".repeat(indent), line)
        };

        match self {
            LogicScriptStatementBody::Comment(comment) => {
                Ok(indent_line(&comment.generate_logic_script(context, ())?))
            }
            LogicScriptStatementBody::Directive(directive) => Ok(indent_line(
                &directive.directive.generate_logic_script(context, ())?,
            )),
            LogicScriptStatementBody::CommandCall(command_call) => Ok(indent_line(
                &command_call.generate_logic_script(context, ())?,
            )),
            LogicScriptStatementBody::IfStatement(if_statement) => {
                let generate_lines = |statements: &[StatementWithOrWithoutLocation<Arg>]| -> Result<Vec<String>, LogicScriptCodeGenerationError> {
                  let mut branch_lines: Vec<String> = statements
                    .iter()
                    .map(|stmt| (*stmt).as_ref().clone())
                    .collect::<Vec<LogicScriptStatement<Arg>>>()
                    .generate_logic_script(context, 2).map(|line| {
                        line.trim_end()
                            .split("\n")
                            .map(|l| format!("{}\n", l))
                            .collect::<Vec<_>>()
                    })?;

                  if branch_lines.len() > 0 && branch_lines[0] == "\n" {
                      branch_lines.remove(0);
                  }

                  Ok(branch_lines)
              };

                let then_lines = generate_lines(&if_statement.then_statements)?;
                let else_lines = generate_lines(&if_statement.else_statements)?;

                let lines = std::iter::once(format!(
                    "if ({}) {{\n",
                    if_statement.conditions.generate_logic_script(context, ())?
                ))
                .chain(then_lines);

                let lines: Box<dyn Iterator<Item = String>> = if else_lines.is_empty() {
                    Box::new(lines.chain(std::iter::once("}".to_string())))
                } else {
                    Box::new(
                        lines
                            .chain(std::iter::once("} else {\n".to_string()))
                            .chain(else_lines)
                            .chain(std::iter::once("}".to_string())),
                    )
                };

                Ok(lines.map(|line| indent_line(&line)).collect::<String>())
            }
            LogicScriptStatementBody::UnaryOperation(unary_operation_statement) => Ok(indent_line(
                &unary_operation_statement.generate_logic_script(context, ())?,
            )),
            LogicScriptStatementBody::ValueAssignment(value_assignment_statement) => Ok(
                indent_line(&value_assignment_statement.generate_logic_script(context, ())?),
            ),
            LogicScriptStatementBody::ArithmeticAssignment(arithmetic_assignment_statement) => Ok(
                indent_line(&arithmetic_assignment_statement.generate_logic_script(context, ())?),
            ),
            LogicScriptStatementBody::LeftIndirectAssignment(
                left_indirect_assignment_statement,
            ) => Ok(indent_line(
                &left_indirect_assignment_statement.generate_logic_script(context, ())?,
            )),
            LogicScriptStatementBody::RightIndirectAssignment(
                right_indirect_assignment_statement,
            ) => Ok(indent_line(
                &right_indirect_assignment_statement.generate_logic_script(context, ())?,
            )),
        }
    }
}

impl<Arg: LogicArgument + GenerateLogicAsm + Clone> GenerateLogicScript
    for LogicScriptStatement<Arg>
{
    type Options = usize; // indentation level

    fn generate_logic_script(
        &self,
        context: &LogicScriptCodeGenerationContext<'_>,
        indent: Self::Options,
    ) -> Result<String, LogicScriptCodeGenerationError> {
        let label_content = match &self.label {
            Some(label) => {
                // When a label precedes an if statement, add an extra blank line
                // between the label and the if body, matching the original output
                // where labels were separate statements and the join newline created
                // a blank line before the if statement.
                let trailing = if matches!(self.body, LogicScriptStatementBody::IfStatement(_)) {
                    "\n\n"
                } else {
                    "\n"
                };
                format!(
                    "\n{}{}:{}",
                    " ".repeat((indent.saturating_sub(2)).max(0)),
                    label,
                    trailing,
                )
            }
            None => "".to_string(),
        };

        Ok(format!(
            "{}{}",
            label_content,
            self.body.generate_logic_script(context, indent)?
        ))
    }
}

impl<Arg: LogicArgument + GenerateLogicAsm + Clone> GenerateLogicScript
    for Vec<LogicScriptStatement<Arg>>
{
    type Options = usize; // indentation level

    fn generate_logic_script(
        &self,
        context: &LogicScriptCodeGenerationContext<'_>,
        options: Self::Options,
    ) -> Result<String, LogicScriptCodeGenerationError> {
        let mut prev_statement: Option<&LogicScriptStatement<Arg>> = None;
        self.iter()
            .map(|statement| {
                let mut script = statement.generate_logic_script(context, options)?;
                let has_label = statement.label.is_some();
                if !has_label
                    && matches!(statement.body, LogicScriptStatementBody::IfStatement(_))
                    && prev_statement.is_some()
                    && !matches!(
                        prev_statement.map(|s| &s.body),
                        Some(
                            LogicScriptStatementBody::Comment(_)
                                | LogicScriptStatementBody::Directive(_)
                        )
                    )
                {
                    script = format!("\n{}", script);
                }
                prev_statement = Some(statement);
                Ok(script)
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|lines| lines.join("\n"))
    }
}
