import flatMap from 'lodash/flatMap';
import { agiCommandsByName } from '../Types/AGICommands';
import { LogicScriptProgram, LogicScriptStatement } from './LogicScriptParserTypes';

type LogicScriptDiagnosticType = 'UnknownCommandName' | 'WrongNumberOfArguments';

export type LogicDiagnostic = {
  severity: 'warning' | 'error';
  statement: LogicScriptStatement;
  type: LogicScriptDiagnosticType;
  message: string;
};

export function getDiagnosticsForStatement(statement: LogicScriptStatement): LogicDiagnostic[] {
  const diagnostics: LogicDiagnostic[] = [];

  if (statement.type === 'CommandCall') {
    const command = agiCommandsByName[statement.commandName];
    if (command == null) {
      if (statement.commandName !== 'goto') {
        diagnostics.push({
          severity: 'error',
          statement,
          type: 'UnknownCommandName',
          message: `Unknown command name: "${statement.commandName}"`,
        });
      }
    } else {
      if (statement.argumentList.length !== command.argTypes.length) {
        diagnostics.push({
          severity: 'error',
          statement,
          type: 'WrongNumberOfArguments',
          message: `Wrong number of arguments: expected ${command.argTypes.length}, got ${statement.argumentList.length}`,
        });
      }
    }
  } else if (statement.type === 'IfStatement') {
    diagnostics.push(
      ...getDiagnosticsForProgram(statement.thenStatements),
      ...getDiagnosticsForProgram(statement.elseStatements),
    );
  }

  return diagnostics;
}

export function getDiagnosticsForProgram(
  program: LogicScriptProgram<LogicScriptStatement>,
): LogicDiagnostic[] {
  return flatMap(program, (statement) => getDiagnosticsForStatement(statement));
}
