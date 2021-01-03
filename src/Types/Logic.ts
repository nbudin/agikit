import { AGICommand, TestCommand } from './AGICommands';

export type LogicCommand = {
  type: 'command';
  address: number;
  agiCommand: AGICommand;
  args: number[];
};

export type LogicTest = {
  type: 'test';
  testCommand: TestCommand;
  args: number[];
  negate: boolean;
};

export type LogicOr = {
  type: 'or';
  orTests: LogicTest[];
};

export type LogicConditionClause = LogicTest | LogicOr;

export type LogicCondition = {
  type: 'condition';
  address: number;
  clauses: LogicConditionClause[];
  skipAddress: number;
};

export type LogicGoto = {
  type: 'goto';
  address: number;
  jumpAddress: number;
};

export type LogicInstruction = LogicCommand | LogicCondition | LogicGoto;

export type LogicLabel = {
  address: number;
  label: string;
  references: LogicInstruction[];
};

export type LogicASTNodeMetadata = {
  instructionAddress?: number;
};

export type LogicCommandNode = LogicCommand & {
  id: string;
  label?: LogicLabel;
  next?: LogicASTNode;
  metadata?: LogicASTNodeMetadata;
};

export type LogicIfNode = {
  type: 'if';
  id: string;
  clauses: LogicConditionClause[];
  then?: LogicASTNode;
  else?: LogicASTNode;
  label?: LogicLabel;
  metadata?: LogicASTNodeMetadata;
};

export type LogicGotoNode = {
  type: 'goto';
  id: string;
  jumpTarget: LogicASTNode;
  label?: LogicLabel;
  metadata?: LogicASTNodeMetadata;
};

export type LogicASTNode = LogicIfNode | LogicGotoNode | LogicCommandNode;

export type LogicResource = {
  instructions: LogicInstruction[];
  messages: (string | undefined)[];
};
