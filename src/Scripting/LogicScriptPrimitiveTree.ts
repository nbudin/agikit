import { LogicScriptPreprocessedStatement } from './LogicScriptParser';
import {
  LogicScriptCommandCall,
  LogicScriptComment,
  LogicScriptIfStatement,
  LogicScriptLabel,
  LogicScriptMessageDirective,
  LogicScriptProgram,
} from './LogicScriptParserTypes';

export type LogicScriptPrimitiveStatement =
  | LogicScriptLabel
  | LogicScriptCommandCall
  | LogicScriptIfStatement<LogicScriptPrimitiveStatement>
  | LogicScriptComment
  | LogicScriptMessageDirective;

function simplifyLogicScriptStatement(
  statement: LogicScriptPreprocessedStatement,
): LogicScriptPrimitiveStatement {
  if (statement.type === 'UnaryOperationStatement') {
    const replacementStatement: LogicScriptCommandCall = {
      type: 'CommandCall',
      commandName: statement.operation === '++' ? 'increment' : 'decrement',
      argumentList: [statement.identifier],
    };
    return replacementStatement;
  }

  if (statement.type === 'ValueAssignmentStatement') {
    const replacementStatement: LogicScriptCommandCall = {
      type: 'CommandCall',
      commandName: statement.value.type === 'Literal' ? 'assignn' : 'assignv',
      argumentList: [statement.assignee, statement.value],
    };
    return replacementStatement;
  }

  if (statement.type === 'ArithmeticAssignmentStatement') {
    let commandFamily: string;
    switch (statement.operator) {
      case '+':
        commandFamily = 'add';
        break;
      case '-':
        commandFamily = 'sub';
        break;
      case '*':
        commandFamily = 'mul.';
        break;
      case '/':
        commandFamily = 'div.';
        break;
    }

    const commandSuffix = statement.value.type === 'Literal' ? 'n' : 'v';

    const replacementStatement: LogicScriptCommandCall = {
      type: 'CommandCall',
      commandName: commandFamily + commandSuffix,
      argumentList: [statement.assignee, statement.value],
    };
    return replacementStatement;
  }

  if (statement.type === 'LeftIndirectAssignmentStatement') {
    const replacementStatement: LogicScriptCommandCall = {
      type: 'CommandCall',
      commandName: statement.value.type === 'Literal' ? 'lindirectn' : 'lindirectv',
      argumentList: [statement.assigneePointer, statement.value],
    };
    return replacementStatement;
  }

  if (statement.type === 'RightIndirectAssignmentStatement') {
    const replacementStatement: LogicScriptCommandCall = {
      type: 'CommandCall',
      commandName: 'rindirect',
      argumentList: [statement.assignee, statement.valuePointer],
    };
    return replacementStatement;
  }

  if (statement.type === 'IfStatement') {
    return {
      ...statement,
      thenStatements: statement.thenStatements.map((s) => simplifyLogicScriptStatement(s)),
      elseStatements: statement.elseStatements.map((s) => simplifyLogicScriptStatement(s)),
    };
  }

  return statement;
}

export function simplifyLogicScriptProgram(
  program: LogicScriptProgram<LogicScriptPreprocessedStatement>,
): LogicScriptProgram<LogicScriptPrimitiveStatement> {
  return program.map((statement) => simplifyLogicScriptStatement(statement));
}
