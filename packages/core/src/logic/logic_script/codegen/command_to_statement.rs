use crate::logic::{
    LogicCommand,
    asm::expressions::{LogicArgument, LogicIdentifier, ParsedLogicArgument},
    logic_script::{
        codegen::{
            context::LogicScriptCodeGenerationContext, errors::LogicScriptCodeGenerationError,
        },
        operators::{LogicScriptArithmeticOperator, LogicScriptUnaryAssignmentOperator},
        statements::{
            LogicScriptArithmeticAssignmentStatement, LogicScriptCommandCall,
            LogicScriptLeftIndirectAssignmentStatement,
            LogicScriptRightIndirectAssignmentStatement, LogicScriptStatement,
            LogicScriptUnaryOperationStatement, LogicScriptValueAssignmentStatement,
        },
    },
};

enum ExpectedArgumentType {
    Any,
    Identifier,
    #[allow(dead_code)]
    Literal,
}

trait CommandStatementTransformer {
    fn command_names(&self) -> &[&str];
    fn expected_argument_types(&self) -> &[ExpectedArgumentType];

    fn transform(
        &self,
        command: &LogicCommand,
        args: &[ParsedLogicArgument],
        context: &LogicScriptCodeGenerationContext,
    ) -> Result<LogicScriptStatement<ParsedLogicArgument>, LogicScriptCodeGenerationError>;

    fn command_name_matches(&self, command: &LogicCommand) -> bool {
        self.command_names()
            .iter()
            .any(|name| command.agi_command.name == *name)
    }

    fn args_match(&self, args: &[ParsedLogicArgument]) -> bool {
        if args.len() != self.expected_argument_types().len() {
            return false;
        }

        args.iter()
            .zip(self.expected_argument_types())
            .all(|(arg, expected_type)| match expected_type {
                ExpectedArgumentType::Any => true,
                ExpectedArgumentType::Identifier => {
                    matches!(arg, ParsedLogicArgument::Identifier(_))
                }
                ExpectedArgumentType::Literal => matches!(arg, ParsedLogicArgument::Literal(_)),
            })
    }

    fn applicable(&self, command: &LogicCommand, args: &[ParsedLogicArgument]) -> bool {
        self.command_name_matches(command) && self.args_match(args)
    }

    fn get_identifier(
        &self,
        arg: &ParsedLogicArgument,
    ) -> Result<LogicIdentifier, LogicScriptCodeGenerationError> {
        match arg {
            ParsedLogicArgument::Identifier(identifier) => Ok(identifier.clone()),
            _ => Err(LogicScriptCodeGenerationError::UnexpectedArgument(
                arg.clone(),
            )),
        }
    }
}

struct IncrementDecrementTransformer;

impl CommandStatementTransformer for IncrementDecrementTransformer {
    fn command_names(&self) -> &'static [&'static str] {
        &["increment", "decrement"]
    }

    fn expected_argument_types(&self) -> &'static [ExpectedArgumentType] {
        &[ExpectedArgumentType::Identifier]
    }

    fn transform(
        &self,
        command: &LogicCommand,
        args: &[ParsedLogicArgument],
        _context: &LogicScriptCodeGenerationContext,
    ) -> Result<LogicScriptStatement<ParsedLogicArgument>, LogicScriptCodeGenerationError> {
        Ok(LogicScriptStatement::UnaryOperation(
            LogicScriptUnaryOperationStatement {
                identifier: self.get_identifier(&args[0])?,
                operation: if command.agi_command.name == "increment" {
                    LogicScriptUnaryAssignmentOperator::Increment
                } else {
                    LogicScriptUnaryAssignmentOperator::Decrement
                },
            },
        ))
    }
}

struct AssignTransformer;

impl CommandStatementTransformer for AssignTransformer {
    fn command_names(&self) -> &'static [&'static str] {
        &["assignn", "assignv"]
    }

    fn expected_argument_types(&self) -> &'static [ExpectedArgumentType] {
        &[ExpectedArgumentType::Identifier, ExpectedArgumentType::Any]
    }

    fn transform(
        &self,
        _command: &LogicCommand,
        args: &[ParsedLogicArgument],
        _context: &LogicScriptCodeGenerationContext,
    ) -> Result<LogicScriptStatement<ParsedLogicArgument>, LogicScriptCodeGenerationError> {
        Ok(LogicScriptStatement::ValueAssignment(
            LogicScriptValueAssignmentStatement {
                assignee: self.get_identifier(&args[0])?,
                value: args[1].clone(),
            },
        ))
    }
}

struct ArithmeticAssignmentTransformer;

impl CommandStatementTransformer for ArithmeticAssignmentTransformer {
    fn command_names(&self) -> &'static [&'static str] {
        &[
            "addn", "addv", "subn", "subv", "mul.n", "mul.v", "div.n", "div.v",
        ]
    }

    fn expected_argument_types(&self) -> &'static [ExpectedArgumentType] {
        &[ExpectedArgumentType::Identifier, ExpectedArgumentType::Any]
    }

    fn transform(
        &self,
        command: &LogicCommand,
        args: &[ParsedLogicArgument],
        _context: &LogicScriptCodeGenerationContext,
    ) -> Result<LogicScriptStatement<ParsedLogicArgument>, LogicScriptCodeGenerationError> {
        let operator = match command.agi_command.name.as_str() {
            "addn" | "addv" => LogicScriptArithmeticOperator::Add,
            "subn" | "subv" => LogicScriptArithmeticOperator::Subtract,
            "mul.n" | "mul.v" => LogicScriptArithmeticOperator::Multiply,
            "div.n" | "div.v" => LogicScriptArithmeticOperator::Divide,
            _ => {
                panic!(
                    "Unexpected command name for arithmetic assignment: {}",
                    command.agi_command.name
                );
            }
        };

        Ok(LogicScriptStatement::ArithmeticAssignment(
            LogicScriptArithmeticAssignmentStatement {
                assignee: self.get_identifier(&args[0])?,
                value: args[1].clone(),
                operator,
            },
        ))
    }
}

struct LeftIndirectAssignmentTransformer;

impl CommandStatementTransformer for LeftIndirectAssignmentTransformer {
    fn command_names(&self) -> &'static [&'static str] {
        &["lindirectn", "lindirectv"]
    }

    fn expected_argument_types(&self) -> &'static [ExpectedArgumentType] {
        &[ExpectedArgumentType::Identifier, ExpectedArgumentType::Any]
    }

    fn transform(
        &self,
        _command: &LogicCommand,
        args: &[ParsedLogicArgument],
        _context: &LogicScriptCodeGenerationContext,
    ) -> Result<LogicScriptStatement<ParsedLogicArgument>, LogicScriptCodeGenerationError> {
        Ok(LogicScriptStatement::LeftIndirectAssignment(
            LogicScriptLeftIndirectAssignmentStatement {
                assignee_pointer: self.get_identifier(&args[0])?,
                value: args[1].clone(),
            },
        ))
    }
}

struct RightIndirectAssignmentTransformer;

impl CommandStatementTransformer for RightIndirectAssignmentTransformer {
    fn command_names(&self) -> &'static [&'static str] {
        &["rindirect"]
    }

    fn expected_argument_types(&self) -> &'static [ExpectedArgumentType] {
        &[
            ExpectedArgumentType::Identifier,
            ExpectedArgumentType::Identifier,
        ]
    }

    fn transform(
        &self,
        _command: &LogicCommand,
        args: &[ParsedLogicArgument],
        _context: &LogicScriptCodeGenerationContext,
    ) -> Result<LogicScriptStatement<ParsedLogicArgument>, LogicScriptCodeGenerationError> {
        Ok(LogicScriptStatement::RightIndirectAssignment(
            LogicScriptRightIndirectAssignmentStatement {
                assignee: self.get_identifier(&args[0])?,
                value_pointer: self.get_identifier(&args[1])?,
            },
        ))
    }
}

const TRANSFORMERS: &[&dyn CommandStatementTransformer] = &[
    &IncrementDecrementTransformer,
    &AssignTransformer,
    &ArithmeticAssignmentTransformer,
    &LeftIndirectAssignmentTransformer,
    &RightIndirectAssignmentTransformer,
];

pub trait CommandToStatement {
    fn to_statement(
        &self,
        context: &LogicScriptCodeGenerationContext,
    ) -> Result<LogicScriptStatement<ParsedLogicArgument>, LogicScriptCodeGenerationError>;
}

impl CommandToStatement for LogicCommand {
    fn to_statement(
        &self,
        context: &LogicScriptCodeGenerationContext,
    ) -> Result<LogicScriptStatement<ParsedLogicArgument>, LogicScriptCodeGenerationError> {
        let command_name = &self.agi_command.name;
        let args = self
            .args()
            .into_iter()
            .map(|asm_arg| asm_arg.try_parse(&context.asm_context))
            .collect::<Result<Vec<_>, _>>()?;

        let transformer = TRANSFORMERS
            .iter()
            .find(|transformer| transformer.applicable(self, &args));

        if let Some(transformer) = transformer {
            transformer.transform(self, &args, context)
        } else {
            Ok(LogicScriptStatement::CommandCall(LogicScriptCommandCall {
                command_name: command_name.clone(),
                argument_list: args,
            }))
        }
    }
}
