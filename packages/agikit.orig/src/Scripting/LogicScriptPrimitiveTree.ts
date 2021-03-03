import {
  LogicScriptCommandCall,
  LogicScriptComment,
  LogicScriptIfStatement,
  LogicScriptLabel,
  LogicScriptMessageDirective,
  LogicScriptProgram,
  LogicScriptStatement,
} from './LogicScriptParserTypes';

export type LogicScriptPrimitiveStatement =
  | LogicScriptLabel
  | LogicScriptCommandCall
  | LogicScriptIfStatement<LogicScriptPrimitiveStatement>
  | LogicScriptComment
  | LogicScriptMessageDirective;

function simplifyLogicScriptStatement(
  statement: LogicScriptStatement,
): LogicScriptPrimitiveStatement {
  if (statement.type === 'UnaryOperationStatement') {
    const replacementStatement: LogicScriptCommandCall = {
      type: 'CommandCall',
      commandName: statement.operation === '++' ? 'increment' : 'decrement',
      argumentList: [statement.identifier],
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
  program: LogicScriptProgram<LogicScriptStatement>,
): LogicScriptProgram<LogicScriptPrimitiveStatement> {
  return program.map((statement) => simplifyLogicScriptStatement(statement));
}
