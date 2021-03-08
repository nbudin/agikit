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
    return { type: 'Comment', comment: comment.map((parts: any[]) => parts[1]).join(''), location: location() };
  }

MultiLineCommentNoLineTerminator
  = "/*" comment:(!("*/" / LineTerminator) SourceCharacter)* "*/" {
    return { type: 'Comment', comment: comment.map((parts: any[]) => parts[1]).join(''), location: location() };
  }

SingleLineComment
  = "//" comment:(!LineTerminator SourceCharacter)* {
    return { type: 'Comment', comment: comment.map((parts: any[]) => parts[1]).join(''), location: location() };
  }

Identifier
  = !Keyword head:IdentifierStart tail:IdentifierPart* {
    return {
      type: 'Identifier',
      name: head + tail.join(""),
      location: location()
    };
  }

IdentifierStart = [A-Za-z_]
IdentifierPart = [A-Za-z0-9_\.]

Keyword
  = IfToken
  / ElseToken

IfToken
  = 'if' !IdentifierPart {
    return {
      type: 'Keyword',
      keyword: 'if',
      location: location(),
    };
  }

ElseToken
  = 'else' !IdentifierPart {
    return {
      type: 'Keyword',
      keyword: 'else',
      location: location(),
    };
  }

Label
  = label:Identifier ':' {
    return {
      type: 'Label',
      label: label.name,
      location: location()
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
  = sign:[+-]? digits:DecimalDigit+ { return { type: "Literal", value: parseInt(text(), 10), location: location() }; }

HexIntegerLiteral
  = "0x"i digits:$HexDigit+ {
      return { type: "Literal", value: parseInt(digits, 16), location: location() };
     }

HexDigit
  = [0-9a-f]i

SingleStringLiteral "string"
  = '"' chars:DoubleStringCharacter* '"' {
      return { type: "Literal", value: chars.join(""), location: location() };
    }
  / "'" chars:SingleStringCharacter* "'" {
      return { type: "Literal", value: chars.join(""), location: location() };
    }

StringLiteral = head:SingleStringLiteral tail:(WSC* SingleStringLiteral)* {
  return {
    type: "Literal",
    value: [head.value, ...tail.map((parts: [{ value: string }, ...string[]]) => parts[0].value)].join(''),
    location: location(),
    stringLocations: [head.location, ...tail.map((parts: [{ location: any }, ...string[]]) => parts[0].location)],
  };
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
      argumentList: argumentList ?? [],
      location: location(),
      commandNameLocation: commandName.location,
    };
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
      argumentList: argumentList ?? [],
      location: location(),
      testNameLocation: testName.location,
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
      location: location(),
    };
  }

SingleBooleanClause = TestCall / BooleanBinaryOperation / ParenthesizedBooleanExpression / NotExpression / Identifier

AndExpression
  = first:SingleBooleanClause WSC* remaining:('&&' WSC* SingleBooleanClause WSC*)+ {
    return {
      type: 'AndExpression',
      clauses: [first, ...remaining.map((parts: any) => parts[2])],
      location: location(),
    };
  }

OrExpression
  = first:SingleBooleanClause WSC* remaining:('||' WSC* SingleBooleanClause WSC*)+ {
    return {
      type: 'OrExpression',
      clauses: [first, ...remaining.map((parts: any) => parts[2])],
      location: location(),
    };
  }

NotExpression
  = '!' expression:SingleBooleanClause {
    return {
      type: 'NotExpression',
      expression,
      location: location(),
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
  = ifKeyword:IfToken WSC* conditions:ParenthesizedBooleanExpression WSC* '{'
    thenStatements:StatementList
    '}'
    elseClause:ElseClause? {
      return {
        type: 'IfStatement',
        conditions,
        thenStatements,
        elseStatements: elseClause?.statements ?? [],
        location: location(),
        ifKeyword,
        elseKeyword: elseClause?.elseKeyword,
      };
    }

ElseClause
  = WSC* elseKeyword:ElseToken WSC* '{' WSC* statements:StatementList? WSC* '}' {
    return { elseKeyword, statements };
  }

MessageDirectiveKeyword = '#message' {
  return {
    type: 'DirectiveKeyword',
    keyword: 'message',
    location: location(),
  };
}

MessageDirective
  = keyword:MessageDirectiveKeyword ' '+ number:DecimalLiteral ' '+ message:StringLiteral {
    return {
      type: 'MessageDirective',
      number,
      message,
      location: location(),
      keyword,
    };
  }

IncludeDirectiveKeyword = '#include' {
  return {
    type: 'DirectiveKeyword',
    keyword: 'include',
    location: location(),
  };
}

IncludeDirective
  = keyword:IncludeDirectiveKeyword ' '+ filename:StringLiteral {
    return {
      type: 'IncludeDirective',
      filename,
      location: location(),
      keyword,
    };
  }

DefineDirectiveKeyword = '#define' {
  return {
    type: 'DirectiveKeyword',
    keyword: 'define',
    location: location(),
  };
}

DefineDirective
  = keyword:DefineDirectiveKeyword ' '+ identifier:Identifier ' '+ value:(Identifier / Literal) {
    return {
      type: 'DefineDirective',
      identifier,
      value,
      location: location(),
      keyword,
    };
  }

UnaryOperationStatement
  = identifier:Identifier WSC* operation:('++' / '--') WSC* ';' {
    return {
      type: 'UnaryOperationStatement',
      identifier,
      operation,
      location: location(),
    };
  }

ValueAssignmentStatement
  = assignee:Identifier WSC* '=' WSC* value:(Identifier / NumericLiteral) ';' {
    return {
      type: 'ValueAssignmentStatement',
      assignee,
      value,
      location: location(),
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
      location: location(),
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
      location: location(),
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
      location: location(),
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
      location: location(),
    };
  }

RightIndirectAssignmentStatement
  = assignee:Identifier WSC* '=' WSC* '*' WSC* valuePointer:Identifier WSC* ';'
  {
    return {
      type: 'RightIndirectAssignmentStatement',
      assignee,
      valuePointer,
      location: location(),
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
