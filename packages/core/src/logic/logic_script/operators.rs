pub trait LogicScriptBinaryOperator {
    fn is_commutative(&self) -> bool;
}

#[derive(Debug, PartialEq, Eq)]
pub enum LogicScriptBooleanBinaryOperator {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
    NotEqual,
}

impl LogicScriptBinaryOperator for LogicScriptBooleanBinaryOperator {
    fn is_commutative(&self) -> bool {
        matches!(
            self,
            LogicScriptBooleanBinaryOperator::Equal | LogicScriptBooleanBinaryOperator::NotEqual
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum LogicScriptUnaryAssignmentOperator {
    Increment,
    Decrement,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LogicScriptArithmeticOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl LogicScriptBinaryOperator for LogicScriptArithmeticOperator {
    fn is_commutative(&self) -> bool {
        matches!(
            self,
            LogicScriptArithmeticOperator::Add | LogicScriptArithmeticOperator::Multiply
        )
    }
}
