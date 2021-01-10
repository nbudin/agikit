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

export type LogicScriptArgumentList = (LogicScriptLiteral | LogicScriptIdentifier)[];

export type LogicScriptCommandCall = {
  type: 'CommandCall';
  commandName: string;
  argumentList: LogicScriptArgumentList;
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
  | LogicScriptTestCall;

export type LogicScriptIfStatement = {
  type: 'IfStatement';
  conditions: LogicScriptBooleanExpression;
  thenStatements: LogicScriptStatement[];
  elseStatements: LogicScriptStatement[];
};

export type LogicScriptStatement =
  | LogicScriptLabel
  | LogicScriptCommandCall
  | LogicScriptIfStatement
  | LogicScriptComment
  | LogicScriptMessageDirective;

export type LogicScriptProgram = LogicScriptStatement[];
