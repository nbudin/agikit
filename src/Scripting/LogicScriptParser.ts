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

  findNextStatementPosition(
    statement: LogicScriptStatement,
    stack: LogicScriptStatementStack,
  ): { index: number; stack: LogicScriptStatementStack } | undefined {
    const statementIndex = stack[0].indexOf(statement);
    if (statementIndex < stack[0].length - 1) {
      return { index: statementIndex + 1, stack };
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
      return this.findNextStatementPosition(enclosingStatement, stack.slice(1));
    }

    return undefined;
  }

  findNextStatement(
    statement: LogicScriptStatement,
    stack: LogicScriptStatementStack,
  ): LogicScriptStatement | undefined {
    const position = this.findNextStatementPosition(statement, stack);
    if (!position) {
      return undefined;
    }

    return position.stack[0][position.index];
  }
}

export function parseLogicScript(source: string): LogicScriptParseTree {
  const parseTree = new LogicScriptParseTree(parse(source));
  const lastStatement = [...parseTree.program]
    .reverse()
    .find((statement) => statement.type === 'CommandCall' || statement.type === 'IfStatement');
  if (
    lastStatement &&
    (lastStatement.type !== 'CommandCall' || lastStatement.commandName !== 'return')
  ) {
    parseTree.program.push({
      type: 'CommandCall',
      commandName: 'return',
      argumentList: [],
    });
  }

  return parseTree;
}

export { SyntaxError };
