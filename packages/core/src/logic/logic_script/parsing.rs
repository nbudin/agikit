use peg::RuleResult;

use crate::logic::{
    asm::{
        expressions::{
            LogicAndExpression, LogicBooleanBinaryOperation, LogicBooleanExpression,
            LogicIdentifier, LogicNotExpression, LogicOrExpression, LogicTestCall,
            ParsedLogicArgument,
        },
        literals::{LogicLiteral, LogicLiteralValue, LogicNumberLiteral},
        operators::LogicBooleanBinaryOperator,
    },
    logic_script::{
        directives::{
            Directive, DirectiveType, LogicScriptDefineValue, LogicScriptDirective,
            LogicScriptDirectiveKeyword,
        },
        literals::{
            LogicScriptLiteral, LogicScriptLiteralValue, LogicScriptSingleStringLiteral,
            LogicScriptStringLiteral, LogicScriptStringLiteralPart,
        },
        locations::{Locatable, WithLocation, location_range},
        operators::{LogicScriptArithmeticOperator, LogicScriptUnaryAssignmentOperator},
        statements::{
            KeywordType, LogicScriptArithmeticAssignmentStatement, LogicScriptCommandCall,
            LogicScriptComment, LogicScriptIfStatement, LogicScriptKeyword, LogicScriptLabel,
            LogicScriptLeftIndirectAssignmentStatement,
            LogicScriptRightIndirectAssignmentStatement, LogicScriptStatement,
            LogicScriptUnaryOperationStatement, LogicScriptValueAssignmentStatement,
        },
    },
};

pub type LogicScriptProgram<StatementType> = Vec<StatementType>;

impl From<SingleBooleanClause> for LogicBooleanExpression<WithLocation<ParsedLogicArgument>> {
    fn from(clause: SingleBooleanClause) -> Self {
        match clause {
            SingleBooleanClause::TestCall(test_call) => {
                LogicBooleanExpression::TestCall(test_call.value)
            }
            SingleBooleanClause::BooleanBinaryOperation(op) => {
                LogicBooleanExpression::BinaryOperation(op.value)
            }
            SingleBooleanClause::ParenthesizedBooleanExpression(expr) => expr.value,
            SingleBooleanClause::NotExpression(not_expr) => {
                LogicBooleanExpression::NotExpression(not_expr.value)
            }
            SingleBooleanClause::Identifier(identifier) => {
                LogicBooleanExpression::Identifier(identifier.value)
            }
        }
    }
}

enum SingleBooleanClause {
    TestCall(WithLocation<LogicTestCall<WithLocation<ParsedLogicArgument>>>),
    BooleanBinaryOperation(
        WithLocation<LogicBooleanBinaryOperation<WithLocation<ParsedLogicArgument>>>,
    ),
    ParenthesizedBooleanExpression(
        WithLocation<LogicBooleanExpression<WithLocation<ParsedLogicArgument>>>,
    ),
    NotExpression(WithLocation<LogicNotExpression<WithLocation<ParsedLogicArgument>>>),
    Identifier(WithLocation<LogicIdentifier>),
}

struct ElseClause {
    statements: Vec<Box<LogicScriptStatement<WithLocation<ParsedLogicArgument>>>>,
    else_keyword: WithLocation<LogicScriptKeyword>,
}

peg::parser! {
    pub grammar logic_script_parser() for str {
        rule source_character() -> char
            = c:[_] { c }

        rule line_terminator() -> ()
            = "\n" / "\r"

        rule white_space() -> ()
            = line_terminator() / " " / "\t" { () }

        rule multi_line_comment() -> WithLocation<LogicScriptComment>
            = start:position!() "/*" comment:$((!"*/" source_character())*) "*/" end:position!() {
                LogicScriptComment {
                    comment: comment.to_string(),
                }.with_location(location_range(start, end))
             }

        rule single_line_comment() -> WithLocation<LogicScriptComment>
            = start:position!() "//" comment:$((!line_terminator() source_character())*) line_terminator() end:position!() {
                LogicScriptComment {
                    comment: comment.to_string(),
                }.with_location(location_range(start, end))
             }

        rule comment() -> WithLocation<LogicScriptComment>
            = multi_line_comment() / single_line_comment()

        rule wsc() -> Option<WithLocation<LogicScriptComment>>
            = comment:comment() { Some(comment) }
            / white_space() { None }

        rule identifier_start() -> char
            = c:(['a'..='z' | 'A'..='Z' | '_']) { c }

        rule identifier_part() -> char
            = c:(['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.']) { c }

        rule identifier() -> WithLocation<LogicIdentifier>
            = start:position!() head:identifier_start() tail:identifier_part()* end:position!() {
                let name = std::iter::once(head)
                    .chain(tail.into_iter())
                    .collect::<String>();
                LogicIdentifier { name }.with_location(location_range(start, end))
            }

        rule if_token()  -> WithLocation<LogicScriptKeyword>
            = start:position!() "if" !identifier_part() end:position!() {
                LogicScriptKeyword {
                    keyword: KeywordType::If
                }.with_location(location_range(start, end))
            }

        rule else_token()  -> WithLocation<LogicScriptKeyword>
            = start:position!() "else" !identifier_part() end:position!() {
                LogicScriptKeyword {
                    keyword: KeywordType::Else,
                }.with_location(location_range(start, end))
            }

        rule keyword() -> WithLocation<LogicScriptKeyword>
            = if_token() / else_token()

        rule label() -> WithLocation<LogicScriptLabel>
            = start:position!() identifier:identifier() ":" end:position!() {
                LogicScriptLabel {
                    label: identifier.value.name
                }.with_location(location_range(start, end))
            }

        rule decimal_digit() -> char
            = c: ['0'..='9'] { c }

        rule decimal_literal() -> WithLocation<LogicNumberLiteral>
            = start:position!() sign:$("+" / "-")? digits:decimal_digit()+ end:position!() {
                let sign_multiplier = if sign == Some("-") { -1 } else { 1 };
                let value = digits.iter().collect::<String>().parse::<i32>().unwrap();
                LogicNumberLiteral {
                    value: value * sign_multiplier
                }.with_location(location_range(start, end))
            }

        rule hex_digit() -> char
            = c: ['0'..='9' | 'a'..='f' | 'A'..='F'] { c }

        rule hex_integer_literal() -> WithLocation<LogicNumberLiteral>
            = start:position!() "0x" digits:$(hex_digit()+) end:position!() {
                let value = i32::from_str_radix(digits, 16).unwrap();
                LogicNumberLiteral {
                    value
                }.with_location(location_range(start, end))
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

        rule single_string_literal() -> WithLocation<LogicScriptSingleStringLiteral>
            = start:position!() "'" chars:(single_string_character()*) "'" end:position!() {
                let value = chars.into_iter().filter_map(|c| c).collect::<String>();
                LogicScriptSingleStringLiteral {
                    value,
                }.with_location(location_range(start, end))
            }
            / start:position!() "\"" chars:(double_string_character()*) "\"" end:position!() {
                let value = chars.into_iter().filter_map(|c| c).collect::<String>();
                LogicScriptSingleStringLiteral {
                    value,
                }.with_location(location_range(start, end))
            }

        rule string_literal_part() -> Option<WithLocation<LogicScriptStringLiteralPart>>
            = wsc:wsc() { wsc.map(|comment| LogicScriptStringLiteralPart::Comment(comment.value).with_location(comment.location)) }
            / string:single_string_literal() { Some(LogicScriptStringLiteralPart::SingleString(string.value).with_location(string.location)) }

        rule string_literal() -> WithLocation<LogicScriptStringLiteral>
            = start:position!() head:single_string_literal() tail:string_literal_part()* end:position!() {
                LogicScriptStringLiteral {
                    parts: std::iter::once(LogicScriptStringLiteralPart::SingleString(head.value))
                        .chain(tail.iter().cloned().filter_map(|part| part.map(|p| p.value)))
                        .collect(),
                }.with_location(location_range(start, end))
            }

        rule numeric_literal() -> WithLocation<LogicNumberLiteral>
            = decimal:decimal_literal() { decimal }
            / hex:hex_integer_literal() { hex }

        rule literal() -> WithLocation<LogicScriptLiteral>
            = number:numeric_literal() {
                let location = number.location.clone();
                LogicScriptLiteral {
                    value: LogicScriptLiteralValue::Number(number.value),
                }.with_location(location)
            }
            / string:string_literal() {
                let location = string.location.clone();
                LogicScriptLiteral {
                    value: LogicScriptLiteralValue::String(string.value),
                }.with_location(location)
        }

        rule argument() -> WithLocation<ParsedLogicArgument>
            = identifier:identifier() { ParsedLogicArgument::Identifier(identifier.value).with_location(identifier.location) }
            / literal:literal() { ParsedLogicArgument::Literal(literal.value.into()).with_location(literal.location) }

        rule argument_list() -> Vec<WithLocation<ParsedLogicArgument>>
            = arguments:(argument() ** ("," wsc()*)) {
                arguments.into_iter().collect()
            }

        rule command_call() -> WithLocation<LogicScriptCommandCall<WithLocation<ParsedLogicArgument>>>
            = start:position!() command_name:identifier() "(" wsc()* argument_list:argument_list()? wsc()* ")" wsc()* ";" end:position!() {
                LogicScriptCommandCall {
                    command_name: command_name.value.name,
                    argument_list: argument_list.unwrap_or_default(),
                }.with_location(location_range(start, end))
            }

        rule test_call() -> WithLocation<LogicTestCall<WithLocation<ParsedLogicArgument>>>
            = start:position!() test_name:identifier() "(" wsc()* argument_list:argument_list()? wsc()* ")" end:position!() {
                LogicTestCall {
                    test_name: test_name.value.name,
                    argument_list: argument_list.unwrap_or_default(),
                }.with_location(location_range(start, end))
            }

        rule boolean_binary_operator() -> LogicBooleanBinaryOperator
            = "<" { LogicBooleanBinaryOperator::LessThan }
            / ">" { LogicBooleanBinaryOperator::GreaterThan }
            / "<=" { LogicBooleanBinaryOperator::LessThanOrEqual }
            / ">=" { LogicBooleanBinaryOperator::GreaterThanOrEqual }
            / "==" { LogicBooleanBinaryOperator::Equal }
            / "!=" { LogicBooleanBinaryOperator::NotEqual }

        rule boolean_binary_operation() -> WithLocation<LogicBooleanBinaryOperation<WithLocation<ParsedLogicArgument>>>
            = start:position!() left:argument() wsc()* operator:boolean_binary_operator() wsc()* right:argument() end:position!() {
                LogicBooleanBinaryOperation {
                    left,
                    operator,
                    right,
                }.with_location(location_range(start, end))
            }

        rule and_expression() -> WithLocation<LogicAndExpression<WithLocation<ParsedLogicArgument>>>
            = start:position!() clauses:(single_boolean_clause() **<2,> (wsc()* "&&" wsc()*)) end:position!() {
                LogicAndExpression {
                    clauses: clauses.into_iter().map(Into::into).collect(),
                }.with_location(location_range(start, end))
            }

        rule or_expression() -> WithLocation<LogicOrExpression<WithLocation<ParsedLogicArgument>>>
            = start:position!() clauses:(single_boolean_clause() **<2,> (wsc()* "||" wsc()*)) end:position!() {
                LogicOrExpression {
                    clauses: clauses.into_iter().map(Into::into).collect(),
                }.with_location(location_range(start, end))
            }

        rule not_expression() -> WithLocation<LogicNotExpression<WithLocation<ParsedLogicArgument>>>
            = start:position!() "!" wsc()* expression:single_boolean_clause() end:position!() {
                LogicNotExpression {
                    expression: Box::new(expression.into()),
                }.with_location(location_range(start, end))
            }

        rule boolean_expression() -> WithLocation<LogicBooleanExpression<WithLocation<ParsedLogicArgument>>>
            = and_expr:and_expression() { LogicBooleanExpression::AndExpression(and_expr.value).with_location(and_expr.location) }
            / or_expr:or_expression() { LogicBooleanExpression::OrExpression(or_expr.value).with_location(or_expr.location) }
            / binary_op:boolean_binary_operation() { LogicBooleanExpression::BinaryOperation(binary_op.value).with_location(binary_op.location) }
            / not_expr:not_expression() { LogicBooleanExpression::NotExpression(not_expr.value).with_location(not_expr.location) }
            / test_call:test_call() { LogicBooleanExpression::TestCall(test_call.value).with_location(test_call.location) }
            / identifier:identifier() { LogicBooleanExpression::Identifier(identifier.value).with_location(identifier.location) }
            / parenthesized_boolean_expression:parenthesized_boolean_expression() { parenthesized_boolean_expression }

        rule parenthesized_boolean_expression() -> WithLocation<LogicBooleanExpression<WithLocation<ParsedLogicArgument>>>
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


        rule if_statement() -> WithLocation<LogicScriptIfStatement<WithLocation<ParsedLogicArgument>, Box<LogicScriptStatement<WithLocation<ParsedLogicArgument>>>>>
            = start:position!() if_keyword:if_token() wsc()* conditions:parenthesized_boolean_expression() wsc()* "{" wsc()* then_statements:statement_list() wsc()* "}" else_clause:else_clause()? end:position!() {
                LogicScriptIfStatement {
                    conditions: conditions.value,
                    if_keyword: if_keyword.value,
                    then_statements,
                    else_keyword: else_clause.as_ref().map(|clause| clause.else_keyword.value.clone()),
                    else_statements: else_clause
                        .map(|clause| clause.statements)
                        .unwrap_or_default(),
                }.with_location(location_range(start, end))
            }

        rule message_directive_keyword() -> WithLocation<LogicScriptDirectiveKeyword>
            = start:position!() "#message" !identifier_part() end:position!() {
                LogicScriptDirectiveKeyword {
                    keyword: DirectiveType::Message,
                }.with_location(location_range(start, end))
            }

        rule include_directive_keyword() -> WithLocation<LogicScriptDirectiveKeyword>
            = start:position!() "#include" !identifier_part() end:position!() {
                LogicScriptDirectiveKeyword {
                    keyword: DirectiveType::Include,
                }.with_location(location_range(start, end))
            }

        rule define_directive_keyword() -> WithLocation<LogicScriptDirectiveKeyword>
            = start:position!() "#define" !identifier_part() end:position!() {
                LogicScriptDirectiveKeyword {
                    keyword: DirectiveType::Define,
                }.with_location(location_range(start, end))
            }

        rule message_directive() -> WithLocation<LogicScriptDirective>
            = start:position!() keyword:message_directive_keyword() " "+ number:decimal_literal() " "+ message:string_literal() end:position!() {
                LogicScriptDirective {
                    keyword: keyword.value,
                    directive: Directive::Message { number: number.value, message: message.value },
                }.with_location(location_range(start, end))
            }

        rule include_directive() -> WithLocation<LogicScriptDirective>
            = start:position!() keyword:include_directive_keyword() " "+ filename:string_literal() end:position!() {
                LogicScriptDirective {
                    keyword: keyword.value,
                    directive: Directive::Include { filename: filename.value },
                }.with_location(location_range(start, end))
            }

        rule define_directive_value() -> WithLocation<LogicScriptDefineValue>
            = identifier:identifier() { LogicScriptDefineValue::Identifier(identifier.value).with_location(identifier.location) }
            / literal:literal() { LogicScriptDefineValue::Literal(literal.value).with_location(literal.location) }

        rule define_directive() -> WithLocation<LogicScriptDirective>
            = start:position!() keyword:define_directive_keyword() " "+ identifier:identifier() " "+ value:define_directive_value() end:position!() {
                LogicScriptDirective {
                    keyword: keyword.value,
                    directive: Directive::Define { identifier: identifier.value, value: value.value },
                }.with_location(location_range(start, end))
            }

        rule unary_operation_statement() -> WithLocation<LogicScriptUnaryOperationStatement>
            = start:position!() identifier:identifier() wsc()* operator:$("++" / "--") wsc()* ";" end:position!() {
                LogicScriptUnaryOperationStatement {
                    operation: if operator == "++" {
                        LogicScriptUnaryAssignmentOperator::Increment
                    } else {
                        LogicScriptUnaryAssignmentOperator::Decrement
                    },
                    identifier: identifier.value,
                }.with_location(location_range(start, end))
            }

        rule numeric_argument() -> WithLocation<ParsedLogicArgument>
            = identifier:identifier() { ParsedLogicArgument::Identifier(identifier.value).with_location(identifier.location) }
            / literal:numeric_literal() {
                ParsedLogicArgument::Literal(LogicLiteral {
                    value: LogicLiteralValue::Number(literal.value),
                }).with_location(literal.location)
            }

        rule value_assignment_statement() -> WithLocation<LogicScriptValueAssignmentStatement<WithLocation<ParsedLogicArgument>>>
            = start:position!() assignee:identifier() wsc()* "=" wsc()* value:numeric_argument() wsc()* ";" end:position!() {
                LogicScriptValueAssignmentStatement {
                    assignee: assignee.value,
                    value,
                }.with_location(location_range(start, end))
            }

        rule arithmetic_operator() -> LogicScriptArithmeticOperator
            = "+" { LogicScriptArithmeticOperator::Add }
            / "-" { LogicScriptArithmeticOperator::Subtract }
            / "*" { LogicScriptArithmeticOperator::Multiply }
            / "/" { LogicScriptArithmeticOperator::Divide }

        rule long_arithmetic_assignment_statement() -> WithLocation<LogicScriptArithmeticAssignmentStatement<WithLocation<ParsedLogicArgument>>>
            = start:position!() assignee:identifier() wsc()* "=" wsc()*
              #{|input, pos| {
                if input[pos..(pos + assignee.value.name.len())] == assignee.value.name {
                    RuleResult::Matched(pos + assignee.value.name.len(), assignee.value.name.clone())
                } else {
                    RuleResult::Failed
                }
              }}
              wsc()* operator:arithmetic_operator() wsc()* value:numeric_argument() wsc()* ";" end:position!() {
                LogicScriptArithmeticAssignmentStatement {
                    operator,
                    assignee: assignee.value,
                    value
                }.with_location(location_range(start, end))
              }
            / start:position!() assignee:identifier() wsc()* "=" wsc()*
              value:numeric_argument() wsc()* operator:arithmetic_operator() wsc()*
              #{|input, pos| {
              if input[pos..(pos + assignee.value.name.len())] == assignee.value.name {
                  RuleResult::Matched(pos + assignee.value.name.len(), assignee.value.name.clone())
              } else {
                  RuleResult::Failed
              }
              }}
              wsc()* ";" end:position!() {
                LogicScriptArithmeticAssignmentStatement {
                    operator,
                    assignee: assignee.value,
                    value,
                }.with_location(location_range(start, end))
              }

        rule short_arithmetic_assignment_statement() -> WithLocation<LogicScriptArithmeticAssignmentStatement<WithLocation<ParsedLogicArgument>>>
            = start:position!() assignee:identifier() wsc()* operator:arithmetic_operator() "=" wsc()* value:numeric_argument() wsc()* ";" end:position!() {
                LogicScriptArithmeticAssignmentStatement {
                    operator,
                    assignee: assignee.value,
                    value,
                }.with_location(location_range(start, end))
            }

        rule arithmetic_assignment_statement() -> WithLocation<LogicScriptArithmeticAssignmentStatement<WithLocation<ParsedLogicArgument>>>
            = long:long_arithmetic_assignment_statement() { long }
            / short:short_arithmetic_assignment_statement() { short }

        rule left_indirect_assignment_statement() -> WithLocation<LogicScriptLeftIndirectAssignmentStatement<WithLocation<ParsedLogicArgument>>>
            = start:position!() "*" wsc()* assignee_pointer:identifier() wsc()* "=" wsc()* value:numeric_argument() wsc()* ";" end:position!() {
                LogicScriptLeftIndirectAssignmentStatement {
                    assignee_pointer: assignee_pointer.value,
                    value,
                }.with_location(location_range(start, end))
            }

        rule right_indirect_assignment_statement() -> WithLocation<LogicScriptRightIndirectAssignmentStatement>
            = start:position!() assignee:identifier() wsc()* "=" wsc()* "*" wsc()* value_pointer:identifier() wsc()* ";" end:position!() {
                LogicScriptRightIndirectAssignmentStatement {
                    assignee: assignee.value,
                    value_pointer: value_pointer.value,
                }.with_location(location_range(start, end))
            }

        rule statement() -> LogicScriptStatement<WithLocation<ParsedLogicArgument>>
            = label:label() { LogicScriptStatement::Label(label.value) }
            / comment:comment() { LogicScriptStatement::Comment(comment.value) }
            / directive:message_directive() { LogicScriptStatement::Directive(directive.value) }
            / directive:include_directive() { LogicScriptStatement::Directive(directive.value) }
            / directive:define_directive() { LogicScriptStatement::Directive(directive.value) }
            / command_call:command_call() { LogicScriptStatement::CommandCall(command_call.value) }
            / if_statement:if_statement() { LogicScriptStatement::IfStatement(if_statement.value) }
            / unary_op:unary_operation_statement() { LogicScriptStatement::UnaryOperation(unary_op.value) }
            / value_assignment:value_assignment_statement() { LogicScriptStatement::ValueAssignment(value_assignment.value) }
            / arithmetic_assignment:arithmetic_assignment_statement() { LogicScriptStatement::ArithmeticAssignment(arithmetic_assignment.value) }
            / left_indirect:left_indirect_assignment_statement() { LogicScriptStatement::LeftIndirectAssignment(left_indirect.value) }
            / right_indirect:right_indirect_assignment_statement() { LogicScriptStatement::RightIndirectAssignment(right_indirect.value) }

        rule statement_list() -> Vec<Box<LogicScriptStatement<WithLocation<ParsedLogicArgument>>>>
            = statements:(statement() ++ (white_space()*)) {
                statements.into_iter().map(Box::new).collect()
            }

        pub rule program() -> LogicScriptProgram<Box<LogicScriptStatement<WithLocation<ParsedLogicArgument>>>>
            = white_space()* statements:statement_list() white_space()* {
                statements
            }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        logic::{
            asm::{
                expressions::{LogicBooleanExpression, ParsedLogicArgument},
                literals::{LogicLiteralValue, LogicNumberLiteral},
            },
            logic_script::{
                locations::WithLocation,
                operators::LogicScriptArithmeticOperator,
                parsing::{LogicScriptProgram, logic_script_parser},
                statements::LogicScriptStatement,
            },
        },
        resources::file_provider::FileProvider,
        test_data::uriquest_dir,
    };

    #[test]
    fn test_parse_command_call() {
        let script = "command(arg1, arg2);";
        let result = logic_script_parser::program(script).expect("Failed to parse script");

        assert_eq!(result.len(), 1, "Expected one statement");
        if let LogicScriptStatement::CommandCall(call) = &*result[0] {
            assert_eq!(call.command_name, "command");
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
                LogicBooleanExpression::OrExpression(_)
            ));
            assert!(if_statement.then_statements.len() == 1);
        } else {
            panic!("Expected an if statement");
        }
    }

    #[test]
    fn test_long_arithmetic_assignments() {
        let expect_correct_results = |result: LogicScriptProgram<
            Box<LogicScriptStatement<WithLocation<ParsedLogicArgument>>>,
        >| {
            if let LogicScriptStatement::ArithmeticAssignment(assignment) = &*result[0] {
                assert!(matches!(
                    assignment.operator,
                    LogicScriptArithmeticOperator::Add
                ));
                assert!(assignment.assignee.name == "v1");
                if let ParsedLogicArgument::Literal(literal) = &assignment.value.value {
                    assert!(matches!(
                        literal.value,
                        LogicLiteralValue::Number(LogicNumberLiteral { value: 2 })
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
        let script = uriquest_dir()
            .read_file_utf8("0.agilogic")
            .expect("Failed to read 0.agilogic file");
        let result = logic_script_parser::program(&script).expect("Failed to parse script");

        assert!(!result.is_empty(), "Parsed script should not be empty");
    }
}
