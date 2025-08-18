use serde::{Deserialize, Serialize};

use crate::logic::{
    LogicConditionClause, LogicTest,
    asm::{
        codegen::{AsmCodeGenerationContext, AsmCodeGenerationError},
        literals::{LogicLiteral, LogicLiteralValue, LogicNumberLiteral, StringLiteral},
        operators::{LogicBinaryOperator, LogicBooleanBinaryOperator},
    },
    commands::AGICommandArgType,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogicIdentifier {
    pub name: String,
}

pub trait LogicArgument {
    fn value_eq(&self, other: &Self) -> bool;
    fn try_parse(
        &self,
        context: &AsmCodeGenerationContext,
    ) -> Result<ParsedLogicArgument, AsmCodeGenerationError>;
}

pub trait AsParsedLogicArgument {
    fn as_parsed(&self) -> &ParsedLogicArgument;
}

#[derive(Debug, Clone)]
pub struct AsmLogicArgument {
    pub value: u16,
    pub arg_type: AGICommandArgType,
}

impl LogicArgument for AsmLogicArgument {
    fn value_eq(&self, other: &Self) -> bool {
        self.value == other.value && self.arg_type == other.arg_type
    }

    fn try_parse(
        &self,
        context: &AsmCodeGenerationContext,
    ) -> Result<ParsedLogicArgument, AsmCodeGenerationError> {
        match self.arg_type {
            AGICommandArgType::Number => Ok(ParsedLogicArgument::Literal(LogicLiteral {
                value: LogicLiteralValue::Number(LogicNumberLiteral {
                    value: self.value.into(),
                }),
            })),
            AGICommandArgType::Variable => Ok(ParsedLogicArgument::Identifier(LogicIdentifier {
                name: format!("v{}", self.value),
            })),
            AGICommandArgType::Flag => Ok(ParsedLogicArgument::Identifier(LogicIdentifier {
                name: format!("f{}", self.value),
            })),
            AGICommandArgType::Object => Ok(ParsedLogicArgument::Identifier(LogicIdentifier {
                name: format!("o{}", self.value),
            })),
            AGICommandArgType::Item => Ok(ParsedLogicArgument::Identifier(LogicIdentifier {
                name: format!("i{}", self.value),
            })),
            AGICommandArgType::String => Ok(ParsedLogicArgument::Identifier(LogicIdentifier {
                name: format!("s{}", self.value),
            })),
            AGICommandArgType::CtrlCode => Ok(ParsedLogicArgument::Identifier(LogicIdentifier {
                name: format!("c{}", self.value),
            })),
            AGICommandArgType::Message => {
                let message = context.logic.messages.get(&(self.value as u8 - 1));
                match message {
                    Some(msg) => Ok(ParsedLogicArgument::Literal(LogicLiteral {
                        value: LogicLiteralValue::from_string(msg.clone()),
                    })),
                    None => Ok(ParsedLogicArgument::Identifier(LogicIdentifier {
                        name: format!("m{}", self.value),
                    })),
                }
            }
            AGICommandArgType::Word => {
                let word = context.word_list.words.get(&(self.value as u16));
                match word {
                    Some(entry) => Ok(ParsedLogicArgument::Literal(LogicLiteral {
                        value: LogicLiteralValue::from_string(entry.canonical_word.clone()),
                    })),
                    None => Ok(ParsedLogicArgument::Identifier(LogicIdentifier {
                        name: format!("w{}", self.value),
                    })),
                }
            }
        }
    }
}

impl AsmLogicArgument {
    pub fn new(value: u16, arg_type: AGICommandArgType) -> Self {
        Self { value, arg_type }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ParsedLogicArgument {
    Literal(LogicLiteral),
    Identifier(LogicIdentifier),
}

impl LogicArgument for ParsedLogicArgument {
    fn value_eq(&self, other: &ParsedLogicArgument) -> bool {
        match (self, other) {
            (ParsedLogicArgument::Identifier(left), ParsedLogicArgument::Identifier(right)) => {
                left.name == right.name
            }
            (ParsedLogicArgument::Literal(left), ParsedLogicArgument::Literal(right)) => {
                match (&left.value, &right.value) {
                    (LogicLiteralValue::Number(left_num), LogicLiteralValue::Number(right_num)) => {
                        left_num.value == right_num.value
                    }
                    (LogicLiteralValue::String(left_str), LogicLiteralValue::String(right_str)) => {
                        left_str.value() == right_str.value()
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn try_parse(
        &self,
        _context: &AsmCodeGenerationContext,
    ) -> Result<ParsedLogicArgument, AsmCodeGenerationError> {
        Ok(self.clone())
    }
}

impl AsParsedLogicArgument for ParsedLogicArgument {
    fn as_parsed(&self) -> &ParsedLogicArgument {
        &self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogicAndExpression<Arg: LogicArgument> {
    pub clauses: Vec<LogicBooleanExpression<Arg>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogicOrExpression<Arg: LogicArgument> {
    pub clauses: Vec<LogicBooleanExpression<Arg>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogicNotExpression<Arg: LogicArgument> {
    pub expression: Box<LogicBooleanExpression<Arg>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogicBooleanBinaryOperation<Arg: LogicArgument> {
    pub left: Arg,
    pub operator: LogicBooleanBinaryOperator,
    pub right: Arg,
}

impl<Arg: LogicArgument> LogicBooleanBinaryOperation<Arg> {
    pub fn logically_equivalent(&self, other: &LogicBooleanBinaryOperation<Arg>) -> bool {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogicTestCall<Arg: LogicArgument> {
    pub test_name: String,
    pub argument_list: Vec<Arg>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogicBooleanExpression<Arg: LogicArgument> {
    BinaryOperation(LogicBooleanBinaryOperation<Arg>),
    AndExpression(LogicAndExpression<Arg>),
    OrExpression(LogicOrExpression<Arg>),
    NotExpression(LogicNotExpression<Arg>),
    TestCall(LogicTestCall<Arg>),
    Identifier(LogicIdentifier),
}

impl<Arg: LogicArgument + AsParsedLogicArgument> LogicBooleanExpression<Arg> {
    pub fn to_parsed(&self) -> LogicBooleanExpression<ParsedLogicArgument> {
        match self {
            LogicBooleanExpression::BinaryOperation(expr) => {
                LogicBooleanExpression::BinaryOperation(LogicBooleanBinaryOperation {
                    left: expr.left.as_parsed().clone(),
                    operator: expr.operator.clone(),
                    right: expr.right.as_parsed().clone(),
                })
            }
            LogicBooleanExpression::AndExpression(expr) => {
                LogicBooleanExpression::AndExpression(LogicAndExpression {
                    clauses: expr.clauses.iter().map(|c| c.to_parsed()).collect(),
                })
            }
            LogicBooleanExpression::OrExpression(expr) => {
                LogicBooleanExpression::OrExpression(LogicOrExpression {
                    clauses: expr.clauses.iter().map(|c| c.to_parsed()).collect(),
                })
            }
            LogicBooleanExpression::NotExpression(expr) => {
                LogicBooleanExpression::NotExpression(LogicNotExpression {
                    expression: Box::new(expr.expression.to_parsed()),
                })
            }
            LogicBooleanExpression::TestCall(expr) => {
                LogicBooleanExpression::TestCall(LogicTestCall {
                    test_name: expr.test_name.clone(),
                    argument_list: expr
                        .argument_list
                        .iter()
                        .map(|arg| arg.as_parsed().clone())
                        .collect(),
                })
            }
            LogicBooleanExpression::Identifier(expr) => {
                LogicBooleanExpression::Identifier(expr.clone())
            }
        }
    }
}

impl LogicBooleanExpression<ParsedLogicArgument> {
    pub fn from_clauses(
        clauses: &[LogicConditionClause],
        context: &AsmCodeGenerationContext,
    ) -> Result<Self, AsmCodeGenerationError> {
        if clauses.len() > 1 {
            return Ok(LogicBooleanExpression::AndExpression(LogicAndExpression {
                clauses: clauses
                    .iter()
                    .map(|clause| LogicBooleanExpression::from_clauses(&[clause.clone()], context))
                    .collect::<Result<_, _>>()?,
            }));
        }

        let clause = clauses.get(0).unwrap();
        match clause {
            LogicConditionClause::Test(test_clause) => {
                let argument_list: Vec<_> = test_clause
                    .args
                    .iter()
                    .zip(if test_clause.test_command.var_args {
                        Box::new(
                            std::iter::repeat(AGICommandArgType::Word).take(test_clause.args.len()),
                        )
                            as Box<dyn std::iter::Iterator<Item = AGICommandArgType>>
                    } else {
                        Box::new(test_clause.test_command.arg_types.iter().copied())
                            as Box<dyn std::iter::Iterator<Item = AGICommandArgType>>
                    })
                    .map(|(value, arg_type)| {
                        AsmLogicArgument::new(*value, arg_type).try_parse(context)
                    })
                    .collect::<Result<_, _>>()?;

                if test_clause.test_command.name == "equaln"
                    || test_clause.test_command.name == "equalv"
                {
                    if argument_list.len() == 2 {
                        return Ok(LogicBooleanExpression::BinaryOperation(
                            LogicBooleanBinaryOperation {
                                left: argument_list[0].clone(),
                                operator: if test_clause.negate {
                                    LogicBooleanBinaryOperator::NotEqual
                                } else {
                                    LogicBooleanBinaryOperator::Equal
                                },
                                right: argument_list[1].clone(),
                            },
                        ));
                    }
                }

                if test_clause.negate {
                    return Ok(LogicBooleanExpression::NotExpression(LogicNotExpression {
                        expression: Box::new(LogicBooleanExpression::from_clauses(
                            &[LogicConditionClause::Test(LogicTest {
                                test_command: test_clause.test_command.clone(),
                                args: test_clause.args.clone(),
                                negate: false,
                            })],
                            context,
                        )?),
                    }));
                }

                if test_clause.test_command.name == "lessn"
                    || test_clause.test_command.name == "lessv"
                {
                    if argument_list.len() == 2 {
                        return Ok(LogicBooleanExpression::BinaryOperation(
                            LogicBooleanBinaryOperation {
                                left: argument_list[0].clone(),
                                operator: LogicBooleanBinaryOperator::LessThan,
                                right: argument_list[1].clone(),
                            },
                        ));
                    }
                }

                if test_clause.test_command.name == "greatern"
                    || test_clause.test_command.name == "greaterv"
                {
                    if argument_list.len() == 2 {
                        return Ok(LogicBooleanExpression::BinaryOperation(
                            LogicBooleanBinaryOperation {
                                left: argument_list[0].clone(),
                                operator: LogicBooleanBinaryOperator::GreaterThan,
                                right: argument_list[1].clone(),
                            },
                        ));
                    }
                }

                Ok(LogicBooleanExpression::TestCall(LogicTestCall {
                    test_name: test_clause.test_command.name.clone(),
                    argument_list,
                }))
            }

            LogicConditionClause::Or(or_clause) => {
                let clauses: Vec<_> = or_clause
                    .or_tests
                    .iter()
                    .map(|test| {
                        LogicBooleanExpression::from_clauses(
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
                            LogicBooleanExpression::BinaryOperation(left),
                            LogicBooleanExpression::BinaryOperation(right),
                        ) => {
                            let args_match = |a: &LogicBooleanBinaryOperation<
                                ParsedLogicArgument,
                            >,
                                              b: &LogicBooleanBinaryOperation<
                                ParsedLogicArgument,
                            >| {
                                if a.left.value_eq(&b.left) && a.right.value_eq(&b.right) {
                                    return true;
                                }

                                if a.operator.is_commutative() || b.operator.is_commutative() {
                                    a.left.value_eq(&b.right) && a.right.value_eq(&b.left)
                                } else {
                                    false
                                }
                            };

                            if (left.operator == LogicBooleanBinaryOperator::LessThan
                                && right.operator == LogicBooleanBinaryOperator::Equal)
                                || (left.operator == LogicBooleanBinaryOperator::Equal
                                    && right.operator == LogicBooleanBinaryOperator::LessThan)
                                    && args_match(left, right)
                            {
                                return Ok(LogicBooleanExpression::BinaryOperation(
                                    LogicBooleanBinaryOperation {
                                        left: left.left.clone(),
                                        operator: LogicBooleanBinaryOperator::LessThanOrEqual,
                                        right: left.right.clone(),
                                    },
                                ));
                            }

                            if (left.operator == LogicBooleanBinaryOperator::GreaterThan
                                && right.operator == LogicBooleanBinaryOperator::Equal)
                                || (left.operator == LogicBooleanBinaryOperator::Equal
                                    && right.operator == LogicBooleanBinaryOperator::GreaterThan)
                                    && args_match(left, right)
                            {
                                return Ok(LogicBooleanExpression::BinaryOperation(
                                    LogicBooleanBinaryOperation {
                                        left: left.left.clone(),
                                        operator: LogicBooleanBinaryOperator::GreaterThanOrEqual,
                                        right: left.right.clone(),
                                    },
                                ));
                            }
                        }
                        _ => {}
                    }
                }

                Ok(LogicBooleanExpression::OrExpression(LogicOrExpression {
                    clauses,
                }))
            }
        }
    }
}
