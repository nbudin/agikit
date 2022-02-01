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

export type LogicProgram = {
  instructions: LogicInstruction[];
  messages: (string | undefined)[];
};
