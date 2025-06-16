use crate::logic::{
    asm::codegen::{AsmCodeGenerationContext, AsmCodeGenerationError},
    commands::AGICommandArgType,
    logic_script::{
        literals::{LogicScriptLiteral, LogicScriptLiteralValue, LogicScriptNumberLiteral},
        operators::{LogicScriptBinaryOperator, LogicScriptBooleanBinaryOperator},
        parsing::ScriptLocationRange,
    },
    LogicConditionClause, LogicTest,
};

#[derive(Debug, Clone)]
pub struct LogicScriptIdentifier {
    pub name: String,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug, Clone)]
pub enum LogicScriptArgument {
    Literal(LogicScriptLiteral),
    Identifier(LogicScriptIdentifier),
}

impl LogicScriptArgument {
    pub fn new(
        value: u8,
        arg_type: AGICommandArgType,
        context: &AsmCodeGenerationContext,
    ) -> Result<Self, AsmCodeGenerationError> {
        match arg_type {
            AGICommandArgType::Number => Ok(LogicScriptArgument::Literal(LogicScriptLiteral {
                value: LogicScriptLiteralValue::Number(LogicScriptNumberLiteral {
                    value: value.into(),
                    location: None,
                }),
                location: None,
            })),
            AGICommandArgType::Variable => {
                Ok(LogicScriptArgument::Identifier(LogicScriptIdentifier {
                    name: format!("v{}", value),
                    location: None,
                }))
            }
            AGICommandArgType::Flag => Ok(LogicScriptArgument::Identifier(LogicScriptIdentifier {
                name: format!("f{}", value),
                location: None,
            })),
            AGICommandArgType::Object => {
                Ok(LogicScriptArgument::Identifier(LogicScriptIdentifier {
                    name: format!("o{}", value),
                    location: None,
                }))
            }
            AGICommandArgType::Item => Ok(LogicScriptArgument::Identifier(LogicScriptIdentifier {
                name: format!("i{}", value),
                location: None,
            })),
            AGICommandArgType::String => {
                Ok(LogicScriptArgument::Identifier(LogicScriptIdentifier {
                    name: format!("s{}", value),
                    location: None,
                }))
            }
            AGICommandArgType::CtrlCode => {
                Ok(LogicScriptArgument::Identifier(LogicScriptIdentifier {
                    name: format!("c{}", value),
                    location: None,
                }))
            }
            AGICommandArgType::Message => {
                let message = context.logic.messages.get(&value);
                match message {
                    Some(msg) => Ok(LogicScriptArgument::Literal(LogicScriptLiteral {
                        value: LogicScriptLiteralValue::from_string(msg.clone(), None),
                        location: None,
                    })),
                    None => Ok(LogicScriptArgument::Identifier(LogicScriptIdentifier {
                        name: format!("m{}", value),
                        location: None,
                    })),
                }
            }
            AGICommandArgType::Word => {
                let word = context.word_list.words.get(&(value as u16));
                match word {
                    Some(entry) => Ok(LogicScriptArgument::Literal(LogicScriptLiteral {
                        value: LogicScriptLiteralValue::from_string(
                            entry.canonical_word.clone(),
                            None,
                        ),
                        location: None,
                    })),
                    None => Err(AsmCodeGenerationError::UnknownWord(value as u16)),
                }
            }
        }
    }

    pub fn value_eq(&self, other: &LogicScriptArgument) -> bool {
        match (self, other) {
            (LogicScriptArgument::Identifier(left), LogicScriptArgument::Identifier(right)) => {
                left.name == right.name
            }
            (LogicScriptArgument::Literal(left), LogicScriptArgument::Literal(right)) => {
                match (&left.value, &right.value) {
                    (
                        LogicScriptLiteralValue::Number(left_num),
                        LogicScriptLiteralValue::Number(right_num),
                    ) => left_num.value == right_num.value,
                    (
                        LogicScriptLiteralValue::String(left_str),
                        LogicScriptLiteralValue::String(right_str),
                    ) => left_str.value() == right_str.value(),
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

pub type LogicScriptArgumentList = Vec<LogicScriptArgument>;

#[derive(Debug)]
pub struct LogicScriptAndExpression {
    pub clauses: Vec<LogicScriptBooleanExpression>,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug)]
pub struct LogicScriptOrExpression {
    pub clauses: Vec<LogicScriptBooleanExpression>,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug)]
pub struct LogicScriptNotExpression {
    pub expression: Box<LogicScriptBooleanExpression>,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug)]
pub struct LogicScriptBooleanBinaryOperation {
    pub left: LogicScriptArgument,
    pub operator: LogicScriptBooleanBinaryOperator,
    pub right: LogicScriptArgument,
    pub location: Option<ScriptLocationRange>,
}

impl LogicScriptBooleanBinaryOperation {
    pub fn logically_equivalent(&self, other: &LogicScriptBooleanBinaryOperation) -> bool {
        if self.left.value_eq(&other.left)
            && self.right.value_eq(&other.right)
            && self.operator == other.operator
        {
            return true;
        }

        if self.operator.is_commutative() {
            self.left.value_eq(&other.right)
                && self.right.value_eq(&other.left)
                && self.operator == other.operator
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct LogicScriptTestCall {
    pub test_name: String,
    pub argument_list: LogicScriptArgumentList,
    pub location: Option<ScriptLocationRange>,
    pub test_name_location: Option<ScriptLocationRange>,
}

#[derive(Debug)]
pub enum LogicScriptBooleanExpression {
    BinaryOperation(LogicScriptBooleanBinaryOperation),
    AndExpression(LogicScriptAndExpression),
    OrExpression(LogicScriptOrExpression),
    NotExpression(LogicScriptNotExpression),
    TestCall(LogicScriptTestCall),
    Identifier(LogicScriptIdentifier),
}

impl LogicScriptBooleanExpression {
    pub fn from_clauses(
        clauses: &[LogicConditionClause],
        context: &AsmCodeGenerationContext,
    ) -> Result<Self, AsmCodeGenerationError> {
        if clauses.len() > 1 {
            return Ok(LogicScriptBooleanExpression::AndExpression(
                LogicScriptAndExpression {
                    clauses: clauses
                        .iter()
                        .map(|clause| {
                            LogicScriptBooleanExpression::from_clauses(&[clause.clone()], context)
                        })
                        .collect::<Result<_, _>>()?,
                    location: None,
                },
            ));
        }

        let clause = clauses.get(0).unwrap();
        match clause {
            LogicConditionClause::Test(test_clause) => {
                let argument_list: Vec<_> = test_clause
                    .args
                    .iter()
                    .zip(test_clause.test_command.arg_types.iter())
                    .map(|(value, arg_type)| LogicScriptArgument::new(*value, *arg_type, context))
                    .collect::<Result<_, _>>()?;

                if test_clause.test_command.name == "equaln"
                    || test_clause.test_command.name == "equalv"
                {
                    if argument_list.len() == 2 {
                        return Ok(LogicScriptBooleanExpression::BinaryOperation(
                            LogicScriptBooleanBinaryOperation {
                                left: argument_list[0].clone(),
                                operator: if test_clause.negate {
                                    LogicScriptBooleanBinaryOperator::Equal
                                } else {
                                    LogicScriptBooleanBinaryOperator::NotEqual
                                },
                                right: argument_list[1].clone(),
                                location: None,
                            },
                        ));
                    }
                }

                if test_clause.negate {
                    return Ok(LogicScriptBooleanExpression::NotExpression(
                        LogicScriptNotExpression {
                            expression: Box::new(LogicScriptBooleanExpression::from_clauses(
                                &[LogicConditionClause::Test(LogicTest {
                                    test_command: test_clause.test_command.clone(),
                                    args: test_clause.args.clone(),
                                    negate: false,
                                })],
                                context,
                            )?),
                            location: None,
                        },
                    ));
                }

                if test_clause.test_command.name == "lessn"
                    || test_clause.test_command.name == "lessv"
                {
                    if argument_list.len() == 2 {
                        return Ok(LogicScriptBooleanExpression::BinaryOperation(
                            LogicScriptBooleanBinaryOperation {
                                left: argument_list[0].clone(),
                                operator: LogicScriptBooleanBinaryOperator::LessThan,
                                right: argument_list[1].clone(),
                                location: None,
                            },
                        ));
                    }
                }

                if test_clause.test_command.name == "greatern"
                    || test_clause.test_command.name == "greaterv"
                {
                    if argument_list.len() == 2 {
                        return Ok(LogicScriptBooleanExpression::BinaryOperation(
                            LogicScriptBooleanBinaryOperation {
                                left: argument_list[0].clone(),
                                operator: LogicScriptBooleanBinaryOperator::GreaterThan,
                                right: argument_list[1].clone(),
                                location: None,
                            },
                        ));
                    }
                }

                Ok(LogicScriptBooleanExpression::TestCall(
                    LogicScriptTestCall {
                        test_name: test_clause.test_command.name.clone(),
                        argument_list,
                        location: None,
                        test_name_location: None,
                    },
                ))
            }

            LogicConditionClause::Or(or_clause) => {
                let clauses: Vec<_> = or_clause
                    .or_tests
                    .iter()
                    .map(|test| {
                        LogicScriptBooleanExpression::from_clauses(
                            &[LogicConditionClause::Test(test.clone())],
                            context,
                        )
                    })
                    .collect::<Result<_, _>>()?;

                // TODO: only consolidate if not in standards mode
                if clauses.len() == 2 {
                    let left = &clauses[0];
                    let right = &clauses[1];

                    match (left, right) {
                        (
                            LogicScriptBooleanExpression::BinaryOperation(left),
                            LogicScriptBooleanExpression::BinaryOperation(right),
                        ) => {
                            let args_match =
                            |a: &LogicScriptBooleanBinaryOperation,
                             b: &LogicScriptBooleanBinaryOperation| {
                                if a.left.value_eq(&b.left) && a.right.value_eq(&b.right) {
                                    return true;
                                }

                                if a.operator.is_commutative() || b.operator.is_commutative() {
                                    a.left.value_eq(&b.right) && a.right.value_eq(&b.left)
                                } else {
                                    false
                                }
                            };

                            if (left.operator == LogicScriptBooleanBinaryOperator::LessThan
                                && right.operator == LogicScriptBooleanBinaryOperator::Equal)
                                || (left.operator == LogicScriptBooleanBinaryOperator::Equal
                                    && right.operator == LogicScriptBooleanBinaryOperator::LessThan)
                                    && args_match(left, right)
                            {
                                return Ok(LogicScriptBooleanExpression::BinaryOperation(
                                    LogicScriptBooleanBinaryOperation {
                                        left: left.left.clone(),
                                        operator: LogicScriptBooleanBinaryOperator::LessThanOrEqual,
                                        right: left.right.clone(),
                                        location: None,
                                    },
                                ));
                            }

                            if (left.operator == LogicScriptBooleanBinaryOperator::GreaterThan
                                && right.operator == LogicScriptBooleanBinaryOperator::Equal)
                                || (left.operator == LogicScriptBooleanBinaryOperator::Equal
                                    && right.operator
                                        == LogicScriptBooleanBinaryOperator::GreaterThan)
                                    && args_match(left, right)
                            {
                                return Ok(LogicScriptBooleanExpression::BinaryOperation(
                                    LogicScriptBooleanBinaryOperation {
                                        left: left.left.clone(),
                                        operator:
                                            LogicScriptBooleanBinaryOperator::GreaterThanOrEqual,
                                        right: left.right.clone(),
                                        location: None,
                                    },
                                ));
                            }
                        }
                        _ => {}
                    }
                }

                Ok(LogicScriptBooleanExpression::OrExpression(
                    LogicScriptOrExpression {
                        clauses,
                        location: None,
                    },
                ))
            }
        }
    }
}
