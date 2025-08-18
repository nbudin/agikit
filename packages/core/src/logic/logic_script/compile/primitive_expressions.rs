use crate::logic::{
    asm::{
        expressions::{
            LogicAndExpression, LogicBooleanExpression, LogicIdentifier, LogicOrExpression,
            LogicTestCall, ParsedLogicArgument,
        },
        literals::{LogicLiteral, LogicLiteralValue, LogicNumberLiteral, LogicStringLiteral},
        operators::LogicBooleanBinaryOperator,
    },
    commands::AGICommandArgType,
    logic_script::{
        compile::ast_generator::ASTGenerationError,
        identifiers::{IdentifierMap, IdentifierMapping},
    },
};

fn flip_binary_operator(operator: LogicBooleanBinaryOperator) -> LogicBooleanBinaryOperator {
    match operator {
        LogicBooleanBinaryOperator::LessThan => LogicBooleanBinaryOperator::GreaterThan,
        LogicBooleanBinaryOperator::LessThanOrEqual => {
            LogicBooleanBinaryOperator::GreaterThanOrEqual
        }
        LogicBooleanBinaryOperator::GreaterThan => LogicBooleanBinaryOperator::LessThan,
        LogicBooleanBinaryOperator::GreaterThanOrEqual => {
            LogicBooleanBinaryOperator::LessThanOrEqual
        }
        LogicBooleanBinaryOperator::Equal => LogicBooleanBinaryOperator::Equal,
        LogicBooleanBinaryOperator::NotEqual => LogicBooleanBinaryOperator::NotEqual,
    }
}

#[derive(Debug, Clone)]
pub enum PrimitiveOrClause {
    TestCall(LogicTestCall<ParsedLogicArgument>),
    NotTestCall(LogicTestCall<ParsedLogicArgument>),
}

impl PrimitiveOrClause {
    pub fn concatenate_complex_or(clauses: &[PrimitiveAndClause]) -> Vec<PrimitiveOrClause> {
        clauses
            .iter()
            .flat_map(|clause| match clause {
                PrimitiveAndClause::Or(or_clauses) => Box::new(
                    PrimitiveOrClause::concatenate_complex_or(
                        or_clauses
                            .iter()
                            .cloned()
                            .map(PrimitiveAndClause::from)
                            .collect::<Vec<_>>()
                            .as_slice(),
                    )
                    .into_iter(),
                )
                    as Box<dyn Iterator<Item = PrimitiveOrClause>>,
                PrimitiveAndClause::TestCall(test_call) => Box::new(std::iter::once(
                    PrimitiveOrClause::TestCall(test_call.clone()),
                ))
                    as Box<dyn Iterator<Item = PrimitiveOrClause>>,
                PrimitiveAndClause::NotTestCall(test_call) => Box::new(std::iter::once(
                    PrimitiveOrClause::NotTestCall(test_call.clone()),
                ))
                    as Box<dyn Iterator<Item = PrimitiveOrClause>>,
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub enum PrimitiveAndClause {
    TestCall(LogicTestCall<ParsedLogicArgument>),
    NotTestCall(LogicTestCall<ParsedLogicArgument>),
    Or(Vec<PrimitiveOrClause>),
}

impl From<PrimitiveOrClause> for PrimitiveAndClause {
    fn from(value: PrimitiveOrClause) -> Self {
        match value {
            PrimitiveOrClause::TestCall(test_call) => PrimitiveAndClause::TestCall(test_call),
            PrimitiveOrClause::NotTestCall(test_call) => PrimitiveAndClause::NotTestCall(test_call),
        }
    }
}

impl PrimitiveAndClause {
    fn try_from_boolean_operation(
        operator: LogicBooleanBinaryOperator,
        left: &ResolvedBinaryOperand,
        right: &ResolvedBinaryOperand,
    ) -> Result<PrimitiveAndClause, ASTGenerationError> {
        match left {
            ResolvedBinaryOperand::Literal(_) => match right {
                ResolvedBinaryOperand::Literal(_) => {
                    return Err(ASTGenerationError::BooleanOperationCannotHaveTwoLiteralOperands);
                }
                ResolvedBinaryOperand::VariableIdentifier { .. } => {
                    PrimitiveAndClause::try_from_boolean_operation(
                        flip_binary_operator(operator),
                        right,
                        left,
                    )
                }
            },
            ResolvedBinaryOperand::VariableIdentifier { name, .. } => {
                let left_arg =
                    ParsedLogicArgument::Identifier(LogicIdentifier { name: name.clone() });
                let (type_suffix, right_arg) = match right {
                    ResolvedBinaryOperand::Literal(literal) => match &literal.value {
                        LogicLiteralValue::String(_) => {
                            return Err(ASTGenerationError::StringLiteralInBooleanOperation);
                        }
                        &LogicLiteralValue::Number(ref number) => (
                            "n",
                            ParsedLogicArgument::Literal(LogicLiteral {
                                value: LogicLiteralValue::Number(number.clone()),
                            }),
                        ),
                    },
                    ResolvedBinaryOperand::VariableIdentifier { name, .. } => (
                        "v",
                        ParsedLogicArgument::Identifier(LogicIdentifier { name: name.clone() }),
                    ),
                };

                Ok(match operator {
                    LogicBooleanBinaryOperator::Equal => {
                        PrimitiveAndClause::TestCall(LogicTestCall {
                            argument_list: vec![left_arg, right_arg],
                            test_name: format!("equal{type_suffix}"),
                        })
                    }
                    LogicBooleanBinaryOperator::NotEqual => {
                        PrimitiveAndClause::NotTestCall(LogicTestCall {
                            argument_list: vec![left_arg, right_arg],
                            test_name: format!("equal{type_suffix}"),
                        })
                    }
                    LogicBooleanBinaryOperator::LessThan => {
                        PrimitiveAndClause::TestCall(LogicTestCall {
                            argument_list: vec![left_arg, right_arg],
                            test_name: format!("less{type_suffix}"),
                        })
                    }
                    LogicBooleanBinaryOperator::GreaterThan => {
                        PrimitiveAndClause::TestCall(LogicTestCall {
                            argument_list: vec![left_arg, right_arg],
                            test_name: format!("greater{type_suffix}"),
                        })
                    }
                    LogicBooleanBinaryOperator::LessThanOrEqual => PrimitiveAndClause::Or(vec![
                        PrimitiveOrClause::TestCall(LogicTestCall {
                            test_name: format!("less{type_suffix}"),
                            argument_list: vec![left_arg.clone(), right_arg.clone()],
                        }),
                        PrimitiveOrClause::TestCall(LogicTestCall {
                            test_name: format!("equal{type_suffix}"),
                            argument_list: vec![left_arg, right_arg],
                        }),
                    ]),
                    LogicBooleanBinaryOperator::GreaterThanOrEqual => PrimitiveAndClause::Or(vec![
                        PrimitiveOrClause::TestCall(LogicTestCall {
                            test_name: format!("greater{type_suffix}"),
                            argument_list: vec![left_arg.clone(), right_arg.clone()],
                        }),
                        PrimitiveOrClause::TestCall(LogicTestCall {
                            test_name: format!("equal{type_suffix}"),
                            argument_list: vec![left_arg, right_arg],
                        }),
                    ]),
                })
            }
        }
    }

    pub fn concatenate_complex_and(
        clauses: &[PrimitiveBooleanExpression],
    ) -> Vec<PrimitiveAndClause> {
        clauses
            .iter()
            .flat_map(|clause| match clause {
                PrimitiveBooleanExpression::And(and_clauses) => Box::new(
                    PrimitiveAndClause::concatenate_complex_and(
                        and_clauses
                            .iter()
                            .cloned()
                            .map(PrimitiveBooleanExpression::from)
                            .collect::<Vec<_>>()
                            .as_slice(),
                    )
                    .into_iter(),
                )
                    as Box<dyn Iterator<Item = PrimitiveAndClause>>,
                PrimitiveBooleanExpression::TestCall(test_call) => Box::new(std::iter::once(
                    PrimitiveAndClause::TestCall(test_call.clone()),
                ))
                    as Box<dyn Iterator<Item = PrimitiveAndClause>>,
                PrimitiveBooleanExpression::NotTestCall(test_call) => Box::new(std::iter::once(
                    PrimitiveAndClause::NotTestCall(test_call.clone()),
                ))
                    as Box<dyn Iterator<Item = PrimitiveAndClause>>,
                PrimitiveBooleanExpression::Or(primitive_or_clauses) => Box::new(std::iter::once(
                    PrimitiveAndClause::Or(primitive_or_clauses.clone()),
                ))
                    as Box<dyn Iterator<Item = PrimitiveAndClause>>,
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub enum PrimitiveBooleanExpression {
    TestCall(LogicTestCall<ParsedLogicArgument>),
    NotTestCall(LogicTestCall<ParsedLogicArgument>),
    Or(Vec<PrimitiveOrClause>),
    And(Vec<PrimitiveAndClause>),
}

impl From<PrimitiveOrClause> for PrimitiveBooleanExpression {
    fn from(value: PrimitiveOrClause) -> Self {
        match value {
            PrimitiveOrClause::TestCall(test_call) => {
                PrimitiveBooleanExpression::TestCall(test_call)
            }
            PrimitiveOrClause::NotTestCall(test_call) => {
                PrimitiveBooleanExpression::NotTestCall(test_call)
            }
        }
    }
}

impl From<PrimitiveAndClause> for PrimitiveBooleanExpression {
    fn from(value: PrimitiveAndClause) -> Self {
        match value {
            PrimitiveAndClause::TestCall(test_call) => {
                PrimitiveBooleanExpression::TestCall(test_call)
            }
            PrimitiveAndClause::NotTestCall(test_call) => {
                PrimitiveBooleanExpression::NotTestCall(test_call)
            }
            PrimitiveAndClause::Or(clauses) => PrimitiveBooleanExpression::Or(clauses),
        }
    }
}

impl PrimitiveBooleanExpression {
    pub fn try_from_logic_boolean_expression(
        expression: &LogicBooleanExpression<ParsedLogicArgument>,
        identifiers: &IdentifierMap,
    ) -> Result<Self, ASTGenerationError> {
        let binary_expression =
            BinaryExpression::try_from_logic_boolean_expression(expression, identifiers)?;
        Ok(binary_expression.to_primitive_boolean_expression())
    }
}

#[derive(Debug, Clone)]
pub enum BinaryExpression {
    TestCall(LogicTestCall<ParsedLogicArgument>),
    Not(Box<BinaryExpression>),
    Or(Box<BinaryExpression>, Box<BinaryExpression>),
    And(Box<BinaryExpression>, Box<BinaryExpression>),
}

impl From<PrimitiveOrClause> for BinaryExpression {
    fn from(value: PrimitiveOrClause) -> Self {
        Self::from(PrimitiveBooleanExpression::from(value))
    }
}

impl From<PrimitiveAndClause> for BinaryExpression {
    fn from(value: PrimitiveAndClause) -> Self {
        Self::from(PrimitiveBooleanExpression::from(value))
    }
}

impl From<PrimitiveBooleanExpression> for BinaryExpression {
    fn from(value: PrimitiveBooleanExpression) -> Self {
        match value {
            PrimitiveBooleanExpression::TestCall(test_call) => {
                BinaryExpression::TestCall(test_call)
            }
            PrimitiveBooleanExpression::NotTestCall(test_call) => {
                BinaryExpression::Not(Box::new(BinaryExpression::TestCall(test_call)))
            }
            PrimitiveBooleanExpression::Or(clauses) => {
                if clauses.len() == 1 {
                    Self::from(clauses[0].clone())
                } else {
                    BinaryExpression::Or(
                        Box::new(Self::from(clauses[0].clone())),
                        Box::new(Self::from(PrimitiveBooleanExpression::Or(
                            clauses[1..].to_vec(),
                        ))),
                    )
                }
            }
            PrimitiveBooleanExpression::And(clauses) => {
                if clauses.len() == 1 {
                    Self::from(clauses[0].clone())
                } else {
                    BinaryExpression::And(
                        Box::new(Self::from(clauses[0].clone())),
                        Box::new(Self::from(PrimitiveBooleanExpression::And(
                            clauses[1..].to_vec(),
                        ))),
                    )
                }
            }
        }
    }
}

impl BinaryExpression {
    pub fn try_to_irreducible_clause(&self) -> Option<PrimitiveOrClause> {
        match self {
            BinaryExpression::TestCall(test_call) => {
                Some(PrimitiveOrClause::TestCall(test_call.clone()))
            }
            BinaryExpression::Not(expr) => match expr.as_ref() {
                BinaryExpression::TestCall(test_call) => {
                    Some(PrimitiveOrClause::NotTestCall(test_call.clone()))
                }
                BinaryExpression::Not(double_negative_expr) => {
                    double_negative_expr.try_to_irreducible_clause()
                }
                _ => None,
            },
            _ => None,
        }
    }

    pub fn is_irreducible(&self) -> bool {
        self.try_to_irreducible_clause().is_some()
    }

    pub fn try_distribute_or_over_and(&self) -> Option<PrimitiveBooleanExpression> {
        let (other_clause, and_clause1, and_clause2) = match self {
            BinaryExpression::Or(left, right) => match (left.as_ref(), right.as_ref()) {
                (BinaryExpression::And(left_left, left_right), _) => {
                    if let Some(left_left) = left_left.try_to_irreducible_clause()
                        && let Some(left_right) = left_right.try_to_irreducible_clause()
                        && let Some(right) = right.try_to_irreducible_clause()
                    {
                        (right, left_left, left_right)
                    } else {
                        return None;
                    }
                }
                (_, BinaryExpression::And(right_left, right_right)) => {
                    if let Some(left) = left.try_to_irreducible_clause()
                        && let Some(right_left) = right_left.try_to_irreducible_clause()
                        && let Some(right_right) = right_right.try_to_irreducible_clause()
                    {
                        (left, right_left, right_right)
                    } else {
                        return None;
                    }
                }
                _ => return None,
            },
            _ => return None,
        };

        Some(PrimitiveBooleanExpression::And(vec![
            PrimitiveAndClause::Or(vec![and_clause1, other_clause.clone()]),
            PrimitiveAndClause::Or(vec![and_clause2, other_clause]),
        ]))
    }

    pub fn try_concatenate_or(&self) -> Option<PrimitiveBooleanExpression> {
        let (other_clause, or_clause1, or_clause2) = match self {
            BinaryExpression::Or(left, right) => match (left.as_ref(), right.as_ref()) {
                (BinaryExpression::Or(left_left, left_right), _) => {
                    if let Some(left_left) = left_left.try_to_irreducible_clause()
                        && let Some(left_right) = left_right.try_to_irreducible_clause()
                        && let Some(right) = right.try_to_irreducible_clause()
                    {
                        (right, left_left, left_right)
                    } else {
                        return None;
                    }
                }
                (_, BinaryExpression::Or(right_left, right_right)) => {
                    if let Some(left) = left.try_to_irreducible_clause()
                        && let Some(right_left) = right_left.try_to_irreducible_clause()
                        && let Some(right_right) = right_right.try_to_irreducible_clause()
                    {
                        (left, right_left, right_right)
                    } else {
                        return None;
                    }
                }
                _ => return None,
            },
            _ => return None,
        };

        Some(PrimitiveBooleanExpression::Or(vec![
            or_clause1,
            or_clause2,
            other_clause,
        ]))
    }

    pub fn try_concatenate_and(&self) -> Option<PrimitiveBooleanExpression> {
        let (other_clause, and_clause1, and_clause2) = match self {
            BinaryExpression::And(left, right) => match (left.as_ref(), right.as_ref()) {
                (BinaryExpression::And(left_left, left_right), _) => {
                    if let Some(left_left) = left_left.try_to_irreducible_clause()
                        && let Some(left_right) = left_right.try_to_irreducible_clause()
                        && let Some(right) = right.try_to_irreducible_clause()
                    {
                        (right, left_left, left_right)
                    } else {
                        return None;
                    }
                }
                (_, BinaryExpression::And(right_left, right_right)) => {
                    if let Some(left) = left.try_to_irreducible_clause()
                        && let Some(right_left) = right_left.try_to_irreducible_clause()
                        && let Some(right_right) = right_right.try_to_irreducible_clause()
                    {
                        (left, right_left, right_right)
                    } else {
                        return None;
                    }
                }
                _ => return None,
            },
            _ => return None,
        };

        Some(PrimitiveBooleanExpression::And(vec![
            and_clause1.into(),
            and_clause2.into(),
            other_clause.into(),
        ]))
    }

    pub fn try_from_logic_boolean_expression(
        expression: &LogicBooleanExpression<ParsedLogicArgument>,
        identifiers: &IdentifierMap,
    ) -> Result<Self, ASTGenerationError> {
        Ok(match expression {
            LogicBooleanExpression::TestCall(test_call) => Self::TestCall(test_call.clone()),
            LogicBooleanExpression::NotExpression(not_expression) => Self::Not(Box::new(
                Self::try_from_logic_boolean_expression(&not_expression.expression, identifiers)?,
            )),
            LogicBooleanExpression::Identifier(identifier) => Self::TestCall(LogicTestCall {
                test_name: "isset".to_string(),
                argument_list: vec![ParsedLogicArgument::Identifier(identifier.clone())],
            }),
            LogicBooleanExpression::BinaryOperation(operation) => {
                let primitive_and = PrimitiveAndClause::try_from_boolean_operation(
                    operation.operator,
                    &ResolvedBinaryOperand::from_argument(&operation.left, identifiers)?,
                    &ResolvedBinaryOperand::from_argument(&operation.right, identifiers)?,
                )?;
                Self::from(primitive_and)
            }
            LogicBooleanExpression::AndExpression(and_expression) => {
                if and_expression.clauses.len() == 1 {
                    Self::try_from_logic_boolean_expression(
                        &and_expression.clauses[0],
                        identifiers,
                    )?
                } else {
                    BinaryExpression::And(
                        Box::new(Self::try_from_logic_boolean_expression(
                            &and_expression.clauses[0],
                            identifiers,
                        )?),
                        Box::new(Self::try_from_logic_boolean_expression(
                            &LogicBooleanExpression::AndExpression(LogicAndExpression {
                                clauses: and_expression.clauses[1..].to_vec(),
                            }),
                            identifiers,
                        )?),
                    )
                }
            }
            LogicBooleanExpression::OrExpression(or_expression) => {
                if or_expression.clauses.len() == 1 {
                    Self::try_from_logic_boolean_expression(&or_expression.clauses[0], identifiers)?
                } else {
                    BinaryExpression::Or(
                        Box::new(Self::try_from_logic_boolean_expression(
                            &or_expression.clauses[0],
                            identifiers,
                        )?),
                        Box::new(Self::try_from_logic_boolean_expression(
                            &LogicBooleanExpression::OrExpression(LogicOrExpression {
                                clauses: or_expression.clauses[1..].to_vec(),
                            }),
                            identifiers,
                        )?),
                    )
                }
            }
        })
    }

    pub fn to_primitive_boolean_expression(&self) -> PrimitiveBooleanExpression {
        if let Some(primitive_or) = self.try_to_irreducible_clause() {
            return PrimitiveBooleanExpression::from(primitive_or);
        };

        if let Some(distributed) = self.try_distribute_or_over_and() {
            return distributed;
        }

        if let Some(concatenated) = self.try_concatenate_or() {
            return concatenated;
        }

        if let Some(concatenated) = self.try_concatenate_and() {
            return concatenated;
        }

        match self {
            BinaryExpression::Or(left, right) => {
                if let Some(irreducible_left) = left.try_to_irreducible_clause()
                    && let Some(irreducible_right) = right.try_to_irreducible_clause()
                {
                    return PrimitiveBooleanExpression::Or(vec![
                        irreducible_left,
                        irreducible_right,
                    ]);
                }

                let simplified_left = left.to_primitive_boolean_expression();
                let simplified_right = right.to_primitive_boolean_expression();

                match (simplified_left, simplified_right) {
                    (
                        PrimitiveBooleanExpression::Or(or_clauses1),
                        PrimitiveBooleanExpression::Or(or_clauses2),
                    ) => PrimitiveBooleanExpression::Or(PrimitiveOrClause::concatenate_complex_or(
                        or_clauses1
                            .iter()
                            .chain(or_clauses2.iter())
                            .cloned()
                            .map(PrimitiveAndClause::from)
                            .collect::<Vec<_>>()
                            .as_slice(),
                    )),
                    (
                        PrimitiveBooleanExpression::And(and_clauses1),
                        PrimitiveBooleanExpression::And(and_clauses2),
                    ) => PrimitiveBooleanExpression::And(
                        PrimitiveAndClause::concatenate_complex_and(
                            and_clauses1
                                .iter()
                                .chain(and_clauses2.iter())
                                .cloned()
                                .map(PrimitiveBooleanExpression::from)
                                .collect::<Vec<_>>()
                                .as_slice(),
                        ),
                    ),
                    (
                        PrimitiveBooleanExpression::Or(or_clauses),
                        PrimitiveBooleanExpression::And(and_clauses),
                    )
                    | (
                        PrimitiveBooleanExpression::And(and_clauses),
                        PrimitiveBooleanExpression::Or(or_clauses),
                    ) => PrimitiveBooleanExpression::And(
                        and_clauses
                            .iter()
                            .flat_map(|and_clause| {
                                PrimitiveOrClause::concatenate_complex_or(
                                    or_clauses
                                        .iter()
                                        .cloned()
                                        .map(PrimitiveAndClause::from)
                                        .chain(std::iter::once(and_clause.clone()))
                                        .collect::<Vec<_>>()
                                        .as_slice(),
                                )
                            })
                            .map(PrimitiveAndClause::from)
                            .collect(),
                    ),
                    _ => panic!("This should not be reachable"),
                }
            }
            BinaryExpression::And(left, right) => {
                if let Some(irreducible_left) = left.try_to_irreducible_clause()
                    && let Some(irreducible_right) = right.try_to_irreducible_clause()
                {
                    return PrimitiveBooleanExpression::And(vec![
                        irreducible_left.into(),
                        irreducible_right.into(),
                    ]);
                }

                let simplified_left = left.to_primitive_boolean_expression();
                let simplified_right = right.to_primitive_boolean_expression();

                PrimitiveBooleanExpression::And(PrimitiveAndClause::concatenate_complex_and(&[
                    simplified_left,
                    simplified_right,
                ]))
            }
            BinaryExpression::Not(not_clause) => match not_clause.as_ref() {
                BinaryExpression::And(left, right) => {
                    let distributed = BinaryExpression::And(
                        Box::new(BinaryExpression::Not(left.clone())),
                        Box::new(BinaryExpression::Not(right.clone())),
                    );
                    distributed.to_primitive_boolean_expression()
                }
                BinaryExpression::Or(left, right) => {
                    let distributed = BinaryExpression::Or(
                        Box::new(BinaryExpression::Not(left.clone())),
                        Box::new(BinaryExpression::Not(right.clone())),
                    );
                    distributed.to_primitive_boolean_expression()
                }
                _ => panic!("This should not be reachable"),
            },
            _ => panic!("This should not be reachable"),
        }
    }
}

pub enum ResolvedBinaryOperand {
    Literal(LogicLiteral),
    VariableIdentifier {
        name: String,
        number: u16,
        variable_type: AGICommandArgType,
    },
}

impl ResolvedBinaryOperand {
    pub fn from_argument(
        arg: &ParsedLogicArgument,
        identifiers: &IdentifierMap,
    ) -> Result<Self, ASTGenerationError> {
        Ok(match arg {
            ParsedLogicArgument::Literal(literal) => Self::Literal(literal.clone()),
            ParsedLogicArgument::Identifier(identifier) => {
                let Some(mapping) = identifiers.get(&identifier.name) else {
                    return Err(ASTGenerationError::UnknownIdentifier(
                        identifier.name.clone(),
                    ));
                };

                match mapping {
                    IdentifierMapping::Variable {
                        name,
                        number,
                        variable_type,
                    } => ResolvedBinaryOperand::VariableIdentifier {
                        name: name.clone(),
                        number: *number,
                        variable_type: *variable_type,
                    },
                    IdentifierMapping::ConstantString { value, .. } => {
                        ResolvedBinaryOperand::Literal(LogicLiteral {
                            value: LogicLiteralValue::String(LogicStringLiteral {
                                value: value.clone(),
                            }),
                        })
                    }
                    IdentifierMapping::ConstantNumber { value, .. } => {
                        ResolvedBinaryOperand::Literal(LogicLiteral {
                            value: LogicLiteralValue::Number(LogicNumberLiteral {
                                value: *value as i32,
                            }),
                        })
                    }
                }
            }
        })
    }
}
