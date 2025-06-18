pub trait LogicBinaryOperator {
    fn is_commutative(&self) -> bool;
}

#[derive(Debug, PartialEq, Eq)]
pub enum LogicBooleanBinaryOperator {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
    NotEqual,
}

impl LogicBinaryOperator for LogicBooleanBinaryOperator {
    fn is_commutative(&self) -> bool {
        matches!(
            self,
            LogicBooleanBinaryOperator::Equal | LogicBooleanBinaryOperator::NotEqual
        )
    }
}
