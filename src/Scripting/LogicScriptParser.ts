import {
  LogicScriptIdentifier,
  LogicScriptProgram,
  LogicScriptStatement,
} from './LogicScriptParserTypes';
import { parse, SyntaxError } from './LogicScriptParser.generated';

export type LogicScriptStatementVisitor = (
  statement: LogicScriptStatement,
  stack: LogicScriptStatement[][],
) => boolean;

export type LogicScriptStatementStack = LogicScriptStatement[][];

export function getGotoTargetLabel(statement: LogicScriptStatement): string | undefined {
  if (statement.type === 'CommandCall' && statement.commandName === 'goto') {
    const labelIdentifier = statement.argumentList[0] as LogicScriptIdentifier;
    return labelIdentifier.name;
  }
  return undefined;
}

export class LogicScriptParseTree {
  program: LogicScriptProgram;

  constructor(program: LogicScriptProgram) {
    this.program = program;
  }

  dfsStatements(visitor: LogicScriptStatementVisitor): boolean {
    return this.dfsStatementsInner(this.program, visitor, []);
  }

  private dfsStatementsInner(
    statements: LogicScriptStatement[],
    visitor: LogicScriptStatementVisitor,
    previousStack: LogicScriptStatementStack,
  ): boolean {
    let changed = false;
    const stackWithStatements = [statements, ...previousStack];

    [...statements].forEach((statement) => {
      if (visitor(statement, stackWithStatements)) {
        changed = true;
      }
      if (statement.type === 'IfStatement') {
        if (this.dfsStatementsInner(statement.thenStatements, visitor, stackWithStatements)) {
          changed = true;
        }
        if (this.dfsStatementsInner(statement.elseStatements, visitor, stackWithStatements)) {
          changed = true;
        }
      }
    });
    return changed;
  }

  findNextStatement(
    statement: LogicScriptStatement,
    stack: LogicScriptStatementStack,
  ): LogicScriptStatement | undefined {
    const statementIndex = stack[0].indexOf(statement);
    if (statementIndex < stack[0].length - 1) {
      return stack[0][statementIndex + 1];
    }

    if (stack.length > 1) {
      const enclosingStatement = stack[1].find(
        (s) =>
          s.type === 'IfStatement' &&
          (s.thenStatements === stack[0] || s.elseStatements === stack[0]),
      );
      if (!enclosingStatement) {
        throw new Error(`Can't find enclosing statement`);
      }
      return this.findNextStatement(enclosingStatement, stack.slice(1));
    }

    return undefined;
  }
}

export function parseLogicScript(source: string): LogicScriptParseTree {
  return new LogicScriptParseTree(parse(source));
}

export { SyntaxError };
