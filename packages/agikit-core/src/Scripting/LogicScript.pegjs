// Somewhat based on https://github.com/metadevpro/ts-pegjs/blob/master/examples/javascript.pegjs

Program = StatementList

SourceCharacter
  = . { return text(); }

LineTerminator
  = '\n' / '\r'

WhiteSpace
  = ' ' / '\t' / LineTerminator

Comment "comment"
  = MultiLineComment
  / SingleLineComment

WSC = WhiteSpace / Comment

MultiLineComment
  = "/*" comment:(!"*/" SourceCharacter)* "*/" {
    return { type: 'Comment', comment: comment.map((parts: any[]) => parts[1]).join('') };
  }

MultiLineCommentNoLineTerminator
  = "/*" comment:(!("*/" / LineTerminator) SourceCharacter)* "*/" {
    return { type: 'Comment', comment: comment.map((parts: any[]) => parts[1]).join('') };
  }

SingleLineComment
  = "//" comment:(!LineTerminator SourceCharacter)* {
    return { type: 'Comment', comment: comment.map((parts: any[]) => parts[1]).join('') };
  }

Identifier
  = !Keyword head:IdentifierStart tail:IdentifierPart* {
    return {
      type: 'Identifier',
      name: head + tail.join("")
    };
  }

IdentifierStart = [A-Za-z_]
IdentifierPart = [A-Za-z0-9_\.]

Keyword
  = IfToken
  / ElseToken

IfToken
  = 'if' !IdentifierPart

ElseToken
  = 'else' !IdentifierPart

Label
  = label:Identifier ':' {
    return {
      type: 'Label',
      label: label.name
    };
  }

Literal = NumericLiteral / StringLiteral

NumericLiteral "number"
  = literal:HexIntegerLiteral !(IdentifierStart / DecimalDigit) {
      return literal;
    }
  / literal:DecimalLiteral !(IdentifierStart / DecimalDigit) {
      return literal;
    }

DecimalDigit = [0-9]

DecimalLiteral
  = sign:[+-]? digits:DecimalDigit+ { return { type: "Literal", value: parseInt(text(), 10) }; }

HexIntegerLiteral
  = "0x"i digits:$HexDigit+ {
      return { type: "Literal", value: parseInt(digits, 16) };
     }

HexDigit
  = [0-9a-f]i

SingleStringLiteral "string"
  = '"' chars:DoubleStringCharacter* '"' {
      return { type: "Literal", value: chars.join("") };
    }
  / "'" chars:SingleStringCharacter* "'" {
      return { type: "Literal", value: chars.join("") };
    }

StringLiteral = strings:(SingleStringLiteral WSC*)+ {
  return { type: "Literal", value: strings.map((parts: [{ value: string }, ...string[]]) => parts[0].value).join('') };
}

DoubleStringCharacter
  = !('"' / "\\" / LineTerminator) SourceCharacter { return text(); }
  / "\\" sequence:EscapeSequence { return sequence; }
  / LineContinuation

SingleStringCharacter
  = !("'" / "\\" / LineTerminator) SourceCharacter { return text(); }
  / "\\" sequence:EscapeSequence { return sequence; }
  / LineContinuation

LineContinuation
  = "\\" LineTerminator { return ""; }

EscapeSequence
  = CharacterEscapeSequence
  / "0" !DecimalDigit { return "\0"; }
  / HexEscapeSequence

CharacterEscapeSequence
  = SingleEscapeCharacter
  / NonEscapeCharacter

SingleEscapeCharacter
  = "'"
  / '"'
  / "\\"
  / "b"  { return "\b"; }
  / "f"  { return "\f"; }
  / "n"  { return "\n"; }
  / "r"  { return "\r"; }
  / "t"  { return "\t"; }
  / "v"  { return "\v"; }

NonEscapeCharacter
  = !(EscapeCharacter / LineTerminator) SourceCharacter { return text(); }

EscapeCharacter
  = SingleEscapeCharacter
  / DecimalDigit
  / "x"
  / "u"

HexEscapeSequence
  = "x" digits:$(HexDigit HexDigit) {
      return String.fromCharCode(parseInt(digits, 16));
    }

CommandCall
  = commandName:Identifier '(' WSC* argumentList:ArgumentList? WSC* ')' WSC* ';' {
    return {
      type: 'CommandCall',
      commandName: commandName.name,
      argumentList: argumentList ?? []
    }
  }

ArgumentList
  = first:Argument? rest:SubsequentArgument* {
    if (first) {
      return [first, ...rest];
    }

    return [];
  }

Argument
  = Identifier / Literal

SubsequentArgument = WSC* ',' WSC* argument:Argument {
  return argument;
}

TestCall
  = testName:Identifier '(' WSC* argumentList:ArgumentList WSC* ')' {
    return {
      type: 'TestCall',
      testName: testName.name,
      argumentList: argumentList ?? []
    };
  }

BooleanBinaryOperator = '<' / '>' / '<=' / '>=' / '==' / '!='

BooleanBinaryOperation
  = left:Argument WSC* operator:BooleanBinaryOperator WSC* right:Argument {
    return {
      type: 'BooleanBinaryOperation',
      operator,
      left,
      right,
    };
  }

SingleBooleanClause = TestCall / BooleanBinaryOperation / ParenthesizedBooleanExpression / NotExpression / Identifier

AndExpression
  = first:SingleBooleanClause WSC* remaining:('&&' WSC* SingleBooleanClause WSC*)+ {
    return {
      type: 'AndExpression',
      clauses: [first, ...remaining.map((parts: any) => parts[2])]
    };
  }

OrExpression
  = first:SingleBooleanClause WSC* remaining:('||' WSC* SingleBooleanClause WSC*)+ {
    return {
      type: 'OrExpression',
      clauses: [first, ...remaining.map((parts: any) => parts[2])]
    };
  }

NotExpression
  = '!' expression:SingleBooleanClause {
    return {
      type: 'NotExpression',
      expression
    };
  }

BooleanExpression
  = AndExpression
  / OrExpression
  / NotExpression
  / BooleanBinaryOperation
  / TestCall
  / Identifier
  / ParenthesizedBooleanExpression

ParenthesizedBooleanExpression = '(' WSC* expression:BooleanExpression WSC* ')' {
  return expression;
}

IfStatement
  = 'if' WSC* conditions:ParenthesizedBooleanExpression WSC* '{'
    thenStatements:StatementList
    '}'
    elseStatements:ElseClause? {
      return {
        type: 'IfStatement',
        conditions,
        thenStatements,
        elseStatements: elseStatements || []
      };
    }

ElseClause
  = WSC* 'else' WSC* '{' WSC* contents:StatementList? WSC* '}' {
    return contents;
  }

MessageDirective
  = '#message' ' '+ number:DecimalLiteral ' '+ message:StringLiteral {
    return {
      type: 'MessageDirective',
      number: number.value,
      message: message.value
    };
  }

IncludeDirective
  = '#include' ' '+ filename:StringLiteral {
    return {
      type: 'IncludeDirective',
      filename: filename.value,
    };
  }

DefineDirective
  = '#define' ' '+ identifier:Identifier ' '+ value:(Identifier / Literal) {
    return {
      type: 'DefineDirective',
      identifier,
      value,
    };
  }

UnaryOperationStatement
  = identifier:Identifier WSC* operation:('++' / '--') WSC* ';' {
    return {
      type: 'UnaryOperationStatement',
      identifier,
      operation,
    };
  }

ValueAssignmentStatement
  = assignee:Identifier WSC* '=' WSC* value:(Identifier / NumericLiteral) ';' {
    return {
      type: 'ValueAssignmentStatement',
      assignee,
      value,
    };
  }

ArithmeticOperator = '+' / '-' / '*' / '/'

LongArithmeticAssignmentStatementLeft
  = assignee:Identifier WSC* '=' WSC*
    assignee2:Identifier WSC* operator:ArithmeticOperator WSC* value:(Identifier / NumericLiteral) WSC* ';'
  & { assignee.name === assignee2.name }
  {
    return {
      type: 'ArithmeticAssignmentStatement',
      operator,
      assignee,
      value,
    };
  }

LongArithmeticAssignmentStatementRight
  = assignee:Identifier WSC* '=' WSC*
    value:(Identifier / NumericLiteral) WSC* operator:ArithmeticOperator WSC* assignee2:Identifier WSC* ';'
  & { assignee.name === assignee2.name }
  {
    return {
      type: 'ArithmeticAssignmentStatement',
      operator,
      assignee,
      value,
    };
  }

ShortArithmeticAssignmentStatement
  = assignee:Identifier WSC* operator:ArithmeticOperator '=' WSC* value:(Identifier / NumericLiteral) WSC* ';'
  {
    return {
      type: 'ArithmeticAssignmentStatement',
      operator,
      assignee,
      value,
    };
  }

ArithmeticAssignmentStatement = LongArithmeticAssignmentStatementLeft / LongArithmeticAssignmentStatementRight / ShortArithmeticAssignmentStatement

LeftIndirectAssignmentStatement
  = '*' WSC* assigneePointer:Identifier WSC* '=' WSC* value:(Identifier / NumericLiteral) WSC* ';'
  {
    return {
      type: 'LeftIndirectAssignmentStatement',
      assigneePointer,
      value,
    };
  }

RightIndirectAssignmentStatement
  = assignee:Identifier WSC* '=' WSC* '*' WSC* valuePointer:Identifier WSC* ';'
  {
    return {
      type: 'RightIndirectAssignmentStatement',
      assignee,
      valuePointer,
    };
  }

Statement
  = Label
  / CommandCall
  / IfStatement
  / Comment
  / MessageDirective
  / IncludeDirective
  / DefineDirective
  / UnaryOperationStatement
  / ValueAssignmentStatement
  / ArithmeticAssignmentStatement
  / LeftIndirectAssignmentStatement
  / RightIndirectAssignmentStatement

StatementList
  = statements:(WhiteSpace* Statement WhiteSpace*)* {
    return statements.map((statement: any) => statement[1]);
  }
