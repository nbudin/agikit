use std::ops::Range;

use peg::RuleResult;

use crate::logic::logic_script::{
    directives::{
        Directive, DirectiveType, LogicScriptDefineValue, LogicScriptDirective,
        LogicScriptDirectiveKeyword,
    },
    expressions::{
        LogicScriptAndExpression, LogicScriptArgument, LogicScriptArgumentList,
        LogicScriptBooleanBinaryOperation, LogicScriptBooleanExpression, LogicScriptIdentifier,
        LogicScriptNotExpression, LogicScriptOrExpression, LogicScriptTestCall,
    },
    literals::{
        LogicScriptLiteral, LogicScriptLiteralValue, LogicScriptNumberLiteral,
        LogicScriptSingleStringLiteral, LogicScriptStringLiteral, LogicScriptStringLiteralPart,
    },
    operators::{
        LogicScriptArithmeticOperator, LogicScriptBooleanBinaryOperator,
        LogicScriptUnaryAssignmentOperator,
    },
    statements::{
        KeywordType, LogicScriptArithmeticAssignmentStatement, LogicScriptCommandCall,
        LogicScriptComment, LogicScriptIfStatement, LogicScriptKeyword, LogicScriptLabel,
        LogicScriptLeftIndirectAssignmentStatement, LogicScriptRightIndirectAssignmentStatement,
        LogicScriptStatement, LogicScriptUnaryOperationStatement,
        LogicScriptValueAssignmentStatement,
    },
};

#[derive(Debug, Clone, Eq, Ord)]
pub struct ScriptLocation {
    pub offset: usize,
}

impl PartialEq for ScriptLocation {
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset
    }
}

impl PartialOrd for ScriptLocation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.offset.cmp(&other.offset))
    }
}

pub type ScriptLocationRange = Range<ScriptLocation>;

fn location_range(start: usize, end: usize) -> ScriptLocationRange {
    ScriptLocation { offset: start }..ScriptLocation { offset: end }
}

pub type LogicScriptProgram<StatementType> = Vec<StatementType>;

impl From<SingleBooleanClause> for LogicScriptBooleanExpression {
    fn from(clause: SingleBooleanClause) -> Self {
        match clause {
            SingleBooleanClause::TestCall(test_call) => {
                LogicScriptBooleanExpression::TestCall(test_call)
            }
            SingleBooleanClause::BooleanBinaryOperation(op) => {
                LogicScriptBooleanExpression::BinaryOperation(op)
            }
            SingleBooleanClause::ParenthesizedBooleanExpression(expr) => expr,
            SingleBooleanClause::NotExpression(not_expr) => {
                LogicScriptBooleanExpression::NotExpression(not_expr)
            }
            SingleBooleanClause::Identifier(identifier) => {
                LogicScriptBooleanExpression::Identifier(identifier)
            }
        }
    }
}

enum SingleBooleanClause {
    TestCall(LogicScriptTestCall),
    BooleanBinaryOperation(LogicScriptBooleanBinaryOperation),
    ParenthesizedBooleanExpression(LogicScriptBooleanExpression),
    NotExpression(LogicScriptNotExpression),
    Identifier(LogicScriptIdentifier),
}

struct ElseClause {
    statements: Vec<Box<LogicScriptStatement>>,
    else_keyword: LogicScriptKeyword,
}

peg::parser! {
    pub grammar logic_script_parser() for str {
        rule source_character() -> char
            = c:[_] { c }

        rule line_terminator() -> ()
            = "\n" / "\r"

        rule white_space() -> ()
            = line_terminator() / " " / "\t" { () }

        rule multi_line_comment() -> LogicScriptComment
            = start:position!() "/*" comment:$((!"*/" source_character())*) "*/" end:position!() {
                LogicScriptComment {
                    comment: comment.to_string(),
                    location: Some(location_range(start, end))
                }
             }

        rule single_line_comment() -> LogicScriptComment
            = start:position!() "//" comment:$((!line_terminator() source_character())*) line_terminator() end:position!() {
                LogicScriptComment {
                    comment: comment.to_string(),
                    location: Some(location_range(start, end))
                }
             }

        rule comment() -> LogicScriptComment
            = multi_line_comment() / single_line_comment()

        rule wsc() -> Option<LogicScriptComment>
            = comment:comment() { Some(comment) }
            / white_space() { None }

        rule identifier_start() -> char
            = c:(['a'..='z' | 'A'..='Z' | '_']) { c }

        rule identifier_part() -> char
            = c:(['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.']) { c }

        rule identifier() -> LogicScriptIdentifier
            = start:position!() head:identifier_start() tail:identifier_part()* end:position!() {
                let name = std::iter::once(head)
                    .chain(tail.into_iter())
                    .collect::<String>();
                LogicScriptIdentifier { name, location: Some(location_range(start, end)) }
            }

        rule if_token()  -> LogicScriptKeyword
            = start:position!() "if" !identifier_part() end:position!() {
                LogicScriptKeyword {
                    keyword: KeywordType::If,
                    location: Some(location_range(start, end))
                }
            }

        rule else_token()  -> LogicScriptKeyword
            = start:position!() "else" !identifier_part() end:position!() {
                LogicScriptKeyword {
                    keyword: KeywordType::If,
                    location: Some(location_range(start, end))
                }
            }

        rule keyword() -> LogicScriptKeyword
            = if_token() / else_token()

        rule label() -> LogicScriptLabel
            = start:position!() identifier:identifier() ":" end:position!() {
                LogicScriptLabel {
                    label: identifier.name,
                    location: Some(location_range(start, end))
                }
            }

        rule decimal_digit() -> char
            = c: ['0'..='9'] { c }

        rule decimal_literal() -> LogicScriptNumberLiteral
            = start:position!() sign:$("+" / "-")? digits:decimal_digit()+ end:position!() {
                let sign_multiplier = if sign == Some("-") { -1 } else { 1 };
                let value = digits.iter().collect::<String>().parse::<i32>().unwrap();
                LogicScriptNumberLiteral {
                    value: value * sign_multiplier,
                    location: Some(location_range(start, end))
                }
            }

        rule hex_digit() -> char
            = c: ['0'..='9' | 'a'..='f' | 'A'..='F'] { c }

        rule hex_integer_literal() -> LogicScriptNumberLiteral
            = start:position!() "0x" digits:$(hex_digit()+) end:position!() {
                let value = i32::from_str_radix(digits, 16).unwrap();
                LogicScriptNumberLiteral {
                    value,
                    location: Some(location_range(start, end))
                }
            }

        rule line_continuation() -> ()
            = "\\" line_terminator() { () }

        rule single_escape_character() -> char
            = c:(['"' | '\'' | '\\']) { c }
            / "b" { '\u{0008}' } // Backspace
            / "f" { '\u{000C}' } // Form feed
            / "n" { '\n' }
            / "r" { '\r' }
            / "t" { '\t' }
            / "v" { '\u{000B}' } // Vertical tab

        rule escape_character() -> char
            = single_escape_character()
            / decimal_digit()
            / c:['x' | 'u'] { c }

        rule non_escape_character() -> char
            = !escape_character() c:source_character() { c }

        rule character_escape_sequence() -> char
            = single_escape_character() / non_escape_character()

        rule hex_escape_sequence() -> char
            = "x" digits:$(hex_digit() hex_digit()) {
                let value = u8::from_str_radix(digits, 16).unwrap();
                value as char
            }

        rule escape_sequence() -> char
            = character_escape_sequence()
            / hex_escape_sequence()
            / "0" !decimal_digit() { '\0' } // Null character

        rule double_string_character() -> Option<char>
            = !(['"' | '\\'] / line_terminator()) c:source_character() { Some(c) }
            / "\\" sequence:escape_sequence() { Some(sequence) }
            / line_continuation() { None }

        rule single_string_character() -> Option<char>
            = !(['\'' | '\\'] / line_terminator()) c:source_character() { Some(c) }
            / "\\" sequence:escape_sequence() { Some(sequence) }
            / line_continuation() { None }

        rule single_string_literal() -> LogicScriptSingleStringLiteral
            = start:position!() "'" chars:(single_string_character()*) "'" end:position!() {
                let value = chars.into_iter().filter_map(|c| c).collect::<String>();
                LogicScriptSingleStringLiteral {
                    value,
                    location: Some(location_range(start, end))
                }
            }
            / start:position!() "\"" chars:(double_string_character()*) "\"" end:position!() {
                let value = chars.into_iter().filter_map(|c| c).collect::<String>();
                LogicScriptSingleStringLiteral {
                    value,
                    location: Some(location_range(start, end))
                }
            }

        rule string_literal_part() -> Option<LogicScriptStringLiteralPart>
            = wsc:wsc() { wsc.map(LogicScriptStringLiteralPart::Comment) }
            / string:single_string_literal() { Some(LogicScriptStringLiteralPart::SingleString(string)) }

        rule string_literal() -> LogicScriptStringLiteral
            = start:position!() head:single_string_literal() tail:string_literal_part()* end:position!() {
                LogicScriptStringLiteral {
                    parts: std::iter::once(LogicScriptStringLiteralPart::SingleString(head))
                        .chain(tail.iter().cloned().filter_map(|part| part))
                        .collect(),
                    location: Some(location_range(start, end))
                }
            }

        rule numeric_literal() -> LogicScriptNumberLiteral
            = decimal:decimal_literal() { decimal }
            / hex:hex_integer_literal() { hex }

        rule literal() -> LogicScriptLiteral
            = number:numeric_literal() {
                let location = number.location.clone();
                LogicScriptLiteral {
                    value: LogicScriptLiteralValue::Number(number),
                    location,
                }
            }
            / string:string_literal() {
                let location = string.location.clone();
                LogicScriptLiteral {
                    value: LogicScriptLiteralValue::String(string),
                    location,
                }
        }

        rule argument() -> LogicScriptArgument
            = identifier:identifier() { LogicScriptArgument::Identifier(identifier) }
            / literal:literal() { LogicScriptArgument::Literal(literal) }

        rule argument_list() -> LogicScriptArgumentList
            = arguments:(argument() ** ("," wsc()*)) {
                arguments.into_iter().collect()
            }

        rule command_call() -> LogicScriptCommandCall
            = start:position!() command_name:identifier() "(" wsc()* argument_list:argument_list()? wsc()* ")" wsc()* ";" end:position!() {
                LogicScriptCommandCall {
                    commmand_name: command_name.name,
                    argument_list: argument_list.unwrap_or_default(),
                    location: Some(location_range(start, end)),
                    command_name_location: command_name.location,
                }
            }

        rule test_call() -> LogicScriptTestCall
            = start:position!() test_name:identifier() "(" wsc()* argument_list:argument_list()? wsc()* ")" end:position!() {
                LogicScriptTestCall {
                    test_name: test_name.name,
                    argument_list: argument_list.unwrap_or_default(),
                    location: Some(location_range(start, end)),
                    test_name_location: test_name.location,
                }
            }

        rule boolean_binary_operator() -> LogicScriptBooleanBinaryOperator
            = "<" { LogicScriptBooleanBinaryOperator::LessThan }
            / ">" { LogicScriptBooleanBinaryOperator::GreaterThan }
            / "<=" { LogicScriptBooleanBinaryOperator::LessThanOrEqual }
            / ">=" { LogicScriptBooleanBinaryOperator::GreaterThanOrEqual }
            / "==" { LogicScriptBooleanBinaryOperator::Equal }
            / "!=" { LogicScriptBooleanBinaryOperator::NotEqual }

        rule boolean_binary_operation() -> LogicScriptBooleanBinaryOperation
            = start:position!() left:argument() wsc()* operator:boolean_binary_operator() wsc()* right:argument() end:position!() {
                LogicScriptBooleanBinaryOperation {
                    left,
                    operator,
                    right,
                    location: Some(location_range(start, end)),
                }
            }

        rule and_expression() -> LogicScriptAndExpression
            = start:position!() clauses:(single_boolean_clause() **<2,> (wsc()* "&&" wsc()*)) end:position!() {
                LogicScriptAndExpression {
                    clauses: clauses.into_iter().map(Into::into).collect(),
                    location: Some(location_range(start, end)),
                }
            }

        rule or_expression() -> LogicScriptOrExpression
            = start:position!() clauses:(single_boolean_clause() **<2,> (wsc()* "||" wsc()*)) end:position!() {
                LogicScriptOrExpression {
                    clauses: clauses.into_iter().map(Into::into).collect(),
                    location: Some(location_range(start, end)),
                }
            }

        rule not_expression() -> LogicScriptNotExpression
            = start:position!() "!" wsc()* expression:single_boolean_clause() end:position!() {
                LogicScriptNotExpression {
                    expression: Box::new(expression.into()),
                    location: Some(location_range(start, end)),
                }
            }

        rule boolean_expression() -> LogicScriptBooleanExpression
            = and_expr:and_expression() { LogicScriptBooleanExpression::AndExpression(and_expr) }
            / or_expr:or_expression() { LogicScriptBooleanExpression::OrExpression(or_expr) }
            / binary_op:boolean_binary_operation() { LogicScriptBooleanExpression::BinaryOperation(binary_op) }
            / not_expr:not_expression() { LogicScriptBooleanExpression::NotExpression(not_expr) }
            / test_call:test_call() { LogicScriptBooleanExpression::TestCall(test_call) }
            / identifier:identifier() { LogicScriptBooleanExpression::Identifier(identifier) }
            / parenthesized_boolean_expression:parenthesized_boolean_expression() { parenthesized_boolean_expression }

        rule parenthesized_boolean_expression() -> LogicScriptBooleanExpression
            = "(" wsc()* expression:boolean_expression() wsc()* ")" {
                expression
            }

        rule single_boolean_clause() -> SingleBooleanClause
            = test_call:test_call() { SingleBooleanClause::TestCall(test_call) }
            / boolean_binary_operation:boolean_binary_operation() { SingleBooleanClause::BooleanBinaryOperation(boolean_binary_operation) }
            / expression:parenthesized_boolean_expression() { SingleBooleanClause::ParenthesizedBooleanExpression(expression) }
            / not_expression:not_expression() { SingleBooleanClause::NotExpression(not_expression) }
            / identifier:identifier() { SingleBooleanClause::Identifier(identifier) }

        rule else_clause() -> ElseClause
            = wsc()* else_keyword:else_token() wsc()* "{" wsc()* statements:statement_list() wsc()* "}" {
                ElseClause {
                    statements,
                    else_keyword,
                }
            }


        rule if_statement() -> LogicScriptIfStatement<Box<LogicScriptStatement>>
            = start:position!() if_keyword:if_token() wsc()* conditions:parenthesized_boolean_expression() wsc()* "{" wsc()* then_statements:statement_list() wsc()* "}" else_clause:else_clause()? end:position!() {
                LogicScriptIfStatement {
                    conditions: conditions.into(),
                    if_keyword,
                    then_statements,
                    else_keyword: else_clause.as_ref().map(|clause| clause.else_keyword.clone()),
                    else_statements: else_clause
                        .map(|clause| clause.statements)
                        .unwrap_or_default(),
                    location: Some(location_range(start, end)),
                }
            }

        rule message_directive_keyword() -> LogicScriptDirectiveKeyword
            = start:position!() "#message" !identifier_part() end:position!() {
                LogicScriptDirectiveKeyword {
                    keyword: DirectiveType::Message,
                    location: Some(location_range(start, end))
                }
            }

        rule include_directive_keyword() -> LogicScriptDirectiveKeyword
            = start:position!() "#include" !identifier_part() end:position!() {
                LogicScriptDirectiveKeyword {
                    keyword: DirectiveType::Include,
                    location: Some(location_range(start, end))
                }
            }

        rule define_directive_keyword() -> LogicScriptDirectiveKeyword
            = start:position!() "#define" !identifier_part() end:position!() {
                LogicScriptDirectiveKeyword {
                    keyword: DirectiveType::Define,
                    location: Some(location_range(start, end))
                }
            }

        rule message_directive() -> LogicScriptDirective
            = start:position!() keyword:message_directive_keyword() " "+ number:decimal_literal() " "+ message:string_literal() end:position!() {
                LogicScriptDirective {
                    keyword,
                    location: Some(location_range(start, end)),
                    directive: Directive::Message { number, message },
                }
            }

        rule include_directive() -> LogicScriptDirective
            = start:position!() keyword:include_directive_keyword() " "+ filename:string_literal() end:position!() {
                LogicScriptDirective {
                    keyword,
                    location: Some(location_range(start, end)),
                    directive: Directive::Include { filename }
                }
            }

        rule define_directive_value() -> LogicScriptDefineValue
            = identifier:identifier() { LogicScriptDefineValue::Identifier(identifier) }
            / literal:literal() { LogicScriptDefineValue::Literal(literal) }

        rule define_directive() -> LogicScriptDirective
            = start:position!() keyword:define_directive_keyword() " "+ identifier:identifier() " "+ value:define_directive_value() end:position!() {
                LogicScriptDirective {
                    keyword,
                    location: Some(location_range(start, end)),
                    directive: Directive::Define { identifier, value }
                }
            }

        rule unary_operation_statement() -> LogicScriptUnaryOperationStatement
            = start:position!() identifier:identifier() wsc()* operator:$("++" / "--") wsc()* ";" end:position!() {
                LogicScriptUnaryOperationStatement {
                    operation: if operator == "++" {
                        LogicScriptUnaryAssignmentOperator::Increment
                    } else {
                        LogicScriptUnaryAssignmentOperator::Decrement
                    },
                    identifier,
                    location: Some(location_range(start, end)),
                }
            }

        rule numeric_argument() -> LogicScriptArgument
            = identifier:identifier() { LogicScriptArgument::Identifier(identifier) }
            / literal:numeric_literal() {
                LogicScriptArgument::Literal(LogicScriptLiteral {
                    location: literal.location.clone(),
                    value: LogicScriptLiteralValue::Number(literal),
                })
            }

        rule value_assignment_statement() -> LogicScriptValueAssignmentStatement
            = start:position!() assignee:identifier() wsc()* "=" wsc()* value:numeric_argument() wsc()* ";" end:position!() {
                LogicScriptValueAssignmentStatement {
                    assignee,
                    value,
                    location: Some(location_range(start, end)),
                }
            }

        rule arithmetic_operator() -> LogicScriptArithmeticOperator
            = "+" { LogicScriptArithmeticOperator::Add }
            / "-" { LogicScriptArithmeticOperator::Subtract }
            / "*" { LogicScriptArithmeticOperator::Multiply }
            / "/" { LogicScriptArithmeticOperator::Divide }

        rule long_arithmetic_assignment_statement() -> LogicScriptArithmeticAssignmentStatement
            = start:position!() assignee:identifier() wsc()* "=" wsc()*
              #{|input, pos| {
                if input[pos..(pos + assignee.name.len())] == assignee.name {
                    RuleResult::Matched(pos + assignee.name.len(), assignee.name.clone())
                } else {
                    RuleResult::Failed
                }
              }}
              wsc()* operator:arithmetic_operator() wsc()* value:numeric_argument() wsc()* ";" end:position!() {
                LogicScriptArithmeticAssignmentStatement {
                        operator,
                        assignee,
                        value,
                        location: Some(location_range(start, end)),
                    }
              }
            / start:position!() assignee:identifier() wsc()* "=" wsc()*
              value:numeric_argument() wsc()* operator:arithmetic_operator() wsc()*
              #{|input, pos| {
              if input[pos..(pos + assignee.name.len())] == assignee.name {
                  RuleResult::Matched(pos + assignee.name.len(), assignee.name.clone())
              } else {
                  RuleResult::Failed
              }
              }}
              wsc()* ";" end:position!() {
              LogicScriptArithmeticAssignmentStatement {
                      operator,
                      assignee,
                      value,
                      location: Some(location_range(start, end)),
                  }
              }

        rule short_arithmetic_assignment_statement() -> LogicScriptArithmeticAssignmentStatement
            = start:position!() assignee:identifier() wsc()* operator:arithmetic_operator() "=" wsc()* value:numeric_argument() wsc()* ";" end:position!() {
                LogicScriptArithmeticAssignmentStatement {
                    operator,
                    assignee,
                    value,
                    location: Some(location_range(start, end)),
                }
            }

        rule arithmetic_assignment_statement() -> LogicScriptArithmeticAssignmentStatement
            = long:long_arithmetic_assignment_statement() { long }
            / short:short_arithmetic_assignment_statement() { short }

        rule left_indirect_assignment_statement() -> LogicScriptLeftIndirectAssignmentStatement
            = start:position!() "*" wsc()* assignee_pointer:identifier() wsc()* "=" wsc()* value:numeric_argument() wsc()* ";" end:position!() {
                LogicScriptLeftIndirectAssignmentStatement {
                    assignee_pointer,
                    value,
                    location: Some(location_range(start, end)),
                }
            }

        rule right_indirect_assignment_statement() -> LogicScriptRightIndirectAssignmentStatement
            = start:position!() assignee:identifier() wsc()* "=" wsc()* "*" wsc()* value_pointer:identifier() wsc()* ";" end:position!() {
                LogicScriptRightIndirectAssignmentStatement {
                    assignee,
                    value_pointer,
                    location: Some(location_range(start, end)),
                }
            }

        rule statement() -> LogicScriptStatement
            = label:label() { LogicScriptStatement::Label(label) }
            / comment:comment() { LogicScriptStatement::Comment(comment) }
            / directive:message_directive() { LogicScriptStatement::Directive(directive) }
            / directive:include_directive() { LogicScriptStatement::Directive(directive) }
            / directive:define_directive() { LogicScriptStatement::Directive(directive) }
            / command_call:command_call() { LogicScriptStatement::CommandCall(command_call) }
            / if_statement:if_statement() { LogicScriptStatement::IfStatement(if_statement) }
            / unary_op:unary_operation_statement() { LogicScriptStatement::UnaryOperation(unary_op) }
            / value_assignment:value_assignment_statement() { LogicScriptStatement::ValueAssignment(value_assignment) }
            / arithmetic_assignment:arithmetic_assignment_statement() { LogicScriptStatement::ArithmeticAssignment(arithmetic_assignment) }
            / left_indirect:left_indirect_assignment_statement() { LogicScriptStatement::LeftIndirectAssignment(left_indirect) }
            / right_indirect:right_indirect_assignment_statement() { LogicScriptStatement::RightIndirectAssignment(right_indirect) }

        rule statement_list() -> Vec<Box<LogicScriptStatement>>
            = statements:(statement() ++ (white_space()*)) {
                statements.into_iter().map(Box::new).collect()
            }

        pub rule program() -> LogicScriptProgram<Box<LogicScriptStatement>>
            = white_space()* statements:statement_list() white_space()* {
                statements
            }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        logic::logic_script::{
            expressions::{LogicScriptArgument, LogicScriptBooleanExpression},
            literals::{LogicScriptLiteralValue, LogicScriptNumberLiteral},
            operators::LogicScriptArithmeticOperator,
            parsing::{logic_script_parser, LogicScriptProgram},
            statements::LogicScriptStatement,
        },
        TEST_DATA_DIR,
    };

    #[test]
    fn test_parse_command_call() {
        let script = "command(arg1, arg2);";
        let result = logic_script_parser::program(script).expect("Failed to parse script");

        assert_eq!(result.len(), 1, "Expected one statement");
        if let LogicScriptStatement::CommandCall(call) = &*result[0] {
            assert_eq!(call.commmand_name, "command");
            assert_eq!(call.argument_list.len(), 2);
        } else {
            panic!("Expected a command call statement");
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let script = r#"
        if (condition) {
            command(arg1, arg2);
        } else {
            command(arg3);
        }"#;

        let result = logic_script_parser::program(script).expect("Failed to parse script");
        assert_eq!(result.len(), 1, "Expected one statement");
    }

    #[test]
    fn test_parse_multi_clause_boolean() {
        let script = r#"
        if ((a && !b) || c) { command(); }
        "#;
        let result = logic_script_parser::program(script).expect("Failed to parse script");

        assert_eq!(result.len(), 1, "Expected one statement");
        if let LogicScriptStatement::IfStatement(if_statement) = &*result[0] {
            assert!(matches!(
                if_statement.conditions,
                LogicScriptBooleanExpression::OrExpression(_)
            ));
            assert!(if_statement.then_statements.len() == 1);
        } else {
            panic!("Expected an if statement");
        }
    }

    #[test]
    fn test_long_arithmetic_assignments() {
        let expect_correct_results = |result: LogicScriptProgram<Box<LogicScriptStatement>>| {
            if let LogicScriptStatement::ArithmeticAssignment(assignment) = &*result[0] {
                assert!(matches!(
                    assignment.operator,
                    LogicScriptArithmeticOperator::Add
                ));
                assert!(assignment.assignee.name == "v1");
                if let LogicScriptArgument::Literal(literal) = &assignment.value {
                    assert!(matches!(
                        literal.value,
                        LogicScriptLiteralValue::Number(LogicScriptNumberLiteral {
                            value: 2,
                            location: _
                        })
                    ));
                } else {
                    panic!("Expected a numeric literal as value");
                }
            } else {
                panic!("Expected an arithmetic assignment statement");
            }
        };

        let left_assign_result =
            logic_script_parser::program("v1 = v1 + 2;").expect("Failed to parse script");
        expect_correct_results(left_assign_result);

        let right_assign_result =
            logic_script_parser::program("v1 = 2 + v1;").expect("Failed to parse script");
        expect_correct_results(right_assign_result);

        logic_script_parser::program("v1 = v2 + 2;").expect_err("Parsing should fail");
        logic_script_parser::program("v1 = 2 + v2;").expect_err("Parsing should fail");
    }

    #[test]
    fn smoke_test() {
        let script = TEST_DATA_DIR
            .get_file("uriquest/0.agilogic")
            .unwrap()
            .contents_utf8()
            .unwrap();
        let result = logic_script_parser::program(&script).expect("Failed to parse script");

        assert!(!result.is_empty(), "Parsed script should not be empty");
    }
}
