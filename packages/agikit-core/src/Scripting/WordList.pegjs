WordDeclarations = head:(WordDeclaration LineTerminator)* tail:(WordDeclaration LineTerminator?) {
  return [...head, tail].map((parts) => parts[0]);
}

SourceCharacter
  = . { return text(); }

LineTerminator
  = '\n' / '\r'

WhiteSpace
  = ' ' / '\t' / LineTerminator

DecimalDigit = [0-9]

WordNumber
  = digits:DecimalDigit+ { return parseInt(text(), 10); }

BareWord
  = head:BareWordStart tail:BareWordPart* { return head + tail.join(""); }

BareWordStart = [A-Za-z\-_]
BareWordPart = [A-Za-z0-9\-_\.]

StringLiteral "string"
  = '"' chars:DoubleStringCharacter* '"' { return chars.join(""); }
  / "'" chars:SingleStringCharacter* "'" { return chars.join(""); }

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

HexDigit
  = [0-9a-f]i

HexEscapeSequence
  = "x" digits:$(HexDigit HexDigit) {
      return String.fromCharCode(parseInt(digits, 16));
    }

Word = ' '* word:(BareWord / StringLiteral) { return word; }

SynonymList = Word*

WordDeclaration = number:WordNumber ':' synonyms:SynonymList {
  return { number, synonyms };
}
