import { LogicScriptParseTree, LogicScriptPreprocessedStatement } from './LogicScriptParser';
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

export function simplifyLogicScriptStatement(
  statement: LogicScriptPreprocessedStatement,
  parseTree: LogicScriptParseTree<LogicScriptPreprocessedStatement>,
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
    const valueType =
      statement.value.type === 'Literal'
        ? 'Literal'
        : parseTree.identifiers.get(statement.value.name)?.identifierType === 'constant'
        ? 'Literal'
        : 'Identifier';
    const replacementStatement: LogicScriptCommandCall = {
      type: 'CommandCall',
      commandName: valueType === 'Literal' ? 'assignn' : 'assignv',
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
      thenStatements: statement.thenStatements.map((s) =>
        simplifyLogicScriptStatement(s, parseTree),
      ),
      elseStatements: statement.elseStatements.map((s) =>
        simplifyLogicScriptStatement(s, parseTree),
      ),
    };
  }

  return statement;
}

export function simplifyLogicScriptProgram(
  parseTree: LogicScriptParseTree<LogicScriptPreprocessedStatement>,
): LogicScriptProgram<LogicScriptPrimitiveStatement> {
  return parseTree.program.map((statement) => simplifyLogicScriptStatement(statement, parseTree));
}
