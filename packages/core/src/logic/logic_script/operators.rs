#[derive(Debug)]
pub enum LogicScriptBooleanBinaryOperator {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
    NotEqual,
}

#[derive(Debug)]
pub enum LogicScriptUnaryAssignmentOperator {
    Increment,
    Decrement,
}

#[derive(Debug)]
pub enum LogicScriptArithmeticOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}
