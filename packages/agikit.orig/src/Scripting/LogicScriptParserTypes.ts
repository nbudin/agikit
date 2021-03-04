export type LogicScriptComment = {
  type: 'Comment';
  comment: string;
};

export type LogicScriptIdentifier = {
  type: 'Identifier';
  name: string;
};

export type LogicScriptMessageDirective = {
  type: 'MessageDirective';
  number: number;
  message: string;
};

export type LogicScriptLabel = {
  type: 'Label';
  label: string;
};

export type LogicScriptLiteral = {
  type: 'Literal';
  value: number | string;
};

export type LogicScriptArgument = LogicScriptLiteral | LogicScriptIdentifier;

export type LogicScriptArgumentList = LogicScriptArgument[];

export type LogicScriptCommandCall = {
  type: 'CommandCall';
  commandName: string;
  argumentList: LogicScriptArgumentList;
};

export type LogicScriptBooleanBinaryOperation = {
  type: 'BooleanBinaryOperation';
  operator: '<' | '>' | '<=' | '>=' | '==' | '!=';
  left: LogicScriptArgument;
  right: LogicScriptArgument;
};

export type LogicScriptTestCall = {
  type: 'TestCall';
  testName: string;
  argumentList: LogicScriptArgumentList;
};

export type LogicScriptAndExpression = {
  type: 'AndExpression';
  clauses: LogicScriptBooleanExpression[];
};

export type LogicScriptOrExpression = {
  type: 'OrExpression';
  clauses: LogicScriptBooleanExpression[];
};

export type LogicScriptNotExpression = {
  type: 'NotExpression';
  expression: LogicScriptBooleanExpression;
};

export type LogicScriptBooleanExpression =
  | LogicScriptAndExpression
  | LogicScriptOrExpression
  | LogicScriptNotExpression
  | LogicScriptBooleanBinaryOperation
  | LogicScriptTestCall;

export interface LogicScriptIfStatement<StatementType = LogicScriptStatement> {
  type: 'IfStatement';
  conditions: LogicScriptBooleanExpression;
  thenStatements: StatementType[];
  elseStatements: StatementType[];
}

export type LogicScriptUnaryOperationStatement = {
  type: 'UnaryOperationStatement';
  operation: '++' | '--';
  identifier: LogicScriptIdentifier;
};

export type LogicScriptValueAssignmentStatement = {
  type: 'ValueAssignmentStatement';
  assignee: LogicScriptIdentifier;
  value: LogicScriptIdentifier | LogicScriptLiteral;
};

export type LogicScriptArithmeticAssignmentStatement = {
  type: 'ArithmeticAssignmentStatement';
  operator: '+' | '-' | '*' | '/';
  assignee: LogicScriptIdentifier;
  value: LogicScriptIdentifier | LogicScriptLiteral;
};

export type LogicScriptLeftIndirectAssignmentStatement = {
  type: 'LeftIndirectAssignmentStatement';
  assigneePointer: LogicScriptIdentifier;
  value: LogicScriptIdentifier | LogicScriptLiteral;
};

export type LogicScriptRightIndirectAssignmentStatement = {
  type: 'RightIndirectAssignmentStatement';
  assignee: LogicScriptIdentifier;
  valuePointer: LogicScriptIdentifier;
};

export type LogicScriptStatement =
  | LogicScriptLabel
  | LogicScriptCommandCall
  | LogicScriptIfStatement<LogicScriptStatement>
  | LogicScriptComment
  | LogicScriptUnaryOperationStatement
  | LogicScriptMessageDirective
  | LogicScriptValueAssignmentStatement
  | LogicScriptArithmeticAssignmentStatement
  | LogicScriptLeftIndirectAssignmentStatement
  | LogicScriptRightIndirectAssignmentStatement;

export type LogicScriptProgram<StatementType> = StatementType[];
