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

StringLiteral "string"
  = '"' chars:DoubleStringCharacter* '"' {
      return { type: "Literal", value: chars.join("") };
    }
  / "'" chars:SingleStringCharacter* "'" {
      return { type: "Literal", value: chars.join("") };
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
  = commandName:Identifier '(' WhiteSpace* argumentList:ArgumentList? WhiteSpace* ');' {
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

SubsequentArgument = WhiteSpace* ',' WhiteSpace* argument:Argument {
  return argument;
}

TestCall
  = testName:Identifier '(' WhiteSpace* argumentList:ArgumentList WhiteSpace* ')' {
    return {
      type: 'TestCall',
      testName: testName.name,
      argumentList: argumentList ?? []
    };
  }

SingleBooleanClause = TestCall / ParenthesizedBooleanExpression / NotExpression

AndExpression
  = first:SingleBooleanClause WhiteSpace* remaining:('&&' WhiteSpace* SingleBooleanClause WhiteSpace*)+ {
    return {
      type: 'AndExpression',
      clauses: [first, ...remaining.map((parts: any) => parts[2])]
    };
  }

OrExpression
  = first:SingleBooleanClause WhiteSpace* remaining:('||' WhiteSpace* SingleBooleanClause WhiteSpace*)+ {
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
  / TestCall

ParenthesizedBooleanExpression = '(' WhiteSpace* expression:BooleanExpression WhiteSpace* ')' {
  return expression;
}

IfStatement
  = 'if' WhiteSpace* conditions:ParenthesizedBooleanExpression WhiteSpace* '{'
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
  = WhiteSpace* 'else' WhiteSpace* '{' WhiteSpace* contents:StatementList? WhiteSpace* '}' {
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

Statement
  = Label
  / CommandCall
  / IfStatement
  / Comment
  / MessageDirective

StatementList
  = statements:(WhiteSpace* Statement WhiteSpace*)* {
    return statements.map((statement: any) => statement[1]);
  }
