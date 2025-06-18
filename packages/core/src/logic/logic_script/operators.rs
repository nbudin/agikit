use crate::logic::asm::operators::LogicBinaryOperator;

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

impl LogicBinaryOperator for LogicScriptArithmeticOperator {
    fn is_commutative(&self) -> bool {
        matches!(
            self,
            LogicScriptArithmeticOperator::Add | LogicScriptArithmeticOperator::Multiply
        )
    }
}
