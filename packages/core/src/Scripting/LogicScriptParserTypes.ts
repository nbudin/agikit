export type PegJSLocation = {
  offset: number;
  line: number;
  column: number;
};

export type PegJSLocationRange = { start: PegJSLocation; end: PegJSLocation };

export type LogicScriptKeyword = {
  type: 'Keyword';
  keyword: 'if' | 'else';
  location?: PegJSLocationRange;
};

export type LogicScriptDirectiveKeyword<KW extends 'message' | 'include' | 'define'> = {
  type: 'DirectiveKeyword';
  keyword: KW;
  location?: PegJSLocationRange;
};

export type LogicScriptComment = {
  type: 'Comment';
  comment: string;
  location?: PegJSLocationRange;
};

export type LogicScriptIdentifier = {
  type: 'Identifier';
  name: string;
  location?: PegJSLocationRange;
};

export type LogicScriptMessageDirective = {
  type: 'MessageDirective';
  number: LogicScriptLiteral<number>;
  message: LogicScriptLiteral<string>;
  keyword: LogicScriptDirectiveKeyword<'message'>;
  location?: PegJSLocationRange;
};

export type LogicScriptIncludeDirective = {
  type: 'IncludeDirective';
  filename: LogicScriptLiteral<string>;
  keyword: LogicScriptDirectiveKeyword<'include'>;
  location?: PegJSLocationRange;
};

export type LogicScriptDefineDirective = {
  type: 'DefineDirective';
  identifier: LogicScriptIdentifier;
  value: LogicScriptIdentifier | LogicScriptLiteral;
  keyword: LogicScriptDirectiveKeyword<'define'>;
  location?: PegJSLocationRange;
};

export type LogicScriptLabel = {
  type: 'Label';
  label: string;
  location?: PegJSLocationRange;
};

export type LogicScriptLiteral<V extends number | string = number | string> = {
  type: 'Literal';
  value: V;
  location?: PegJSLocationRange;
};

export type LogicScriptArgument = LogicScriptLiteral | LogicScriptIdentifier;

export type LogicScriptArgumentList = LogicScriptArgument[];

export type LogicScriptCommandCall = {
  type: 'CommandCall';
  commandName: string;
  argumentList: LogicScriptArgumentList;
  location?: PegJSLocationRange;
  commandNameLocation?: PegJSLocationRange;
};

export type LogicScriptBooleanBinaryOperation = {
  type: 'BooleanBinaryOperation';
  operator: '<' | '>' | '<=' | '>=' | '==' | '!=';
  left: LogicScriptArgument;
  right: LogicScriptArgument;
  location?: PegJSLocationRange;
};

export type LogicScriptTestCall = {
  type: 'TestCall';
  testName: string;
  argumentList: LogicScriptArgumentList;
  location?: PegJSLocationRange;
  testNameLocation?: PegJSLocationRange;
};

export type LogicScriptAndExpression = {
  type: 'AndExpression';
  clauses: LogicScriptBooleanExpression[];
  location?: PegJSLocationRange;
};

export type LogicScriptOrExpression = {
  type: 'OrExpression';
  clauses: LogicScriptBooleanExpression[];
  location?: PegJSLocationRange;
};

export type LogicScriptNotExpression = {
  type: 'NotExpression';
  expression: LogicScriptBooleanExpression;
  location?: PegJSLocationRange;
};

export type LogicScriptBooleanExpression =
  | LogicScriptAndExpression
  | LogicScriptOrExpression
  | LogicScriptNotExpression
  | LogicScriptBooleanBinaryOperation
  | LogicScriptTestCall
  | LogicScriptIdentifier;

export interface LogicScriptIfStatement<StatementType = LogicScriptStatement> {
  type: 'IfStatement';
  conditions: LogicScriptBooleanExpression;
  thenStatements: StatementType[];
  elseStatements: StatementType[];
  ifKeyword: LogicScriptKeyword;
  elseKeyword?: LogicScriptKeyword;
  location?: PegJSLocationRange;
}

export type LogicScriptUnaryOperationStatement = {
  type: 'UnaryOperationStatement';
  operation: '++' | '--';
  identifier: LogicScriptIdentifier;
  location?: PegJSLocationRange;
};

export type LogicScriptValueAssignmentStatement = {
  type: 'ValueAssignmentStatement';
  assignee: LogicScriptIdentifier;
  value: LogicScriptIdentifier | LogicScriptLiteral;
  location?: PegJSLocationRange;
};

export type LogicScriptArithmeticAssignmentStatement = {
  type: 'ArithmeticAssignmentStatement';
  operator: '+' | '-' | '*' | '/';
  assignee: LogicScriptIdentifier;
  value: LogicScriptIdentifier | LogicScriptLiteral;
  location?: PegJSLocationRange;
};

export type LogicScriptLeftIndirectAssignmentStatement = {
  type: 'LeftIndirectAssignmentStatement';
  assigneePointer: LogicScriptIdentifier;
  value: LogicScriptIdentifier | LogicScriptLiteral;
  location?: PegJSLocationRange;
};

export type LogicScriptRightIndirectAssignmentStatement = {
  type: 'RightIndirectAssignmentStatement';
  assignee: LogicScriptIdentifier;
  valuePointer: LogicScriptIdentifier;
  location?: PegJSLocationRange;
};

export type LogicScriptStatement =
  | LogicScriptLabel
  | LogicScriptCommandCall
  | LogicScriptIfStatement<LogicScriptStatement>
  | LogicScriptComment
  | LogicScriptUnaryOperationStatement
  | LogicScriptMessageDirective
  | LogicScriptIncludeDirective
  | LogicScriptDefineDirective
  | LogicScriptValueAssignmentStatement
  | LogicScriptArithmeticAssignmentStatement
  | LogicScriptLeftIndirectAssignmentStatement
  | LogicScriptRightIndirectAssignmentStatement;

export type LogicScriptProgram<StatementType> = StatementType[];
