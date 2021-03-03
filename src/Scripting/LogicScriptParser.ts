import {
  LogicScriptIdentifier,
  LogicScriptIfStatement,
  LogicScriptProgram,
  LogicScriptStatement,
} from './LogicScriptParserTypes';
import { parse, SyntaxError } from './LogicScriptParser.generated';

export type LogicScriptStatementVisitor<StatementType extends LogicScriptStatement> = (
  statement: StatementType,
  stack: LogicScriptStatementStack<StatementType>,
) => boolean;

export type LogicScriptStatementStack<
  StatementType extends LogicScriptStatement
> = StatementType[][];

export function getGotoTargetLabel(statement: LogicScriptStatement): string | undefined {
  if (statement.type === 'CommandCall' && statement.commandName === 'goto') {
    const labelIdentifier = statement.argumentList[0] as LogicScriptIdentifier;
    return labelIdentifier.name;
  }
  return undefined;
}

export class LogicScriptParseTree<StatementType extends LogicScriptStatement> {
  program: LogicScriptProgram<StatementType>;

  constructor(program: LogicScriptProgram<StatementType>) {
    this.program = program;
  }

  dfsStatements(visitor: LogicScriptStatementVisitor<StatementType>): boolean {
    return this.dfsStatementsInner(this.program, visitor, []);
  }

  private dfsStatementsInner(
    statements: StatementType[],
    visitor: LogicScriptStatementVisitor<StatementType>,
    previousStack: LogicScriptStatementStack<StatementType>,
  ): boolean {
    let changed = false;
    const stackWithStatements = [statements, ...previousStack];

    [...statements].forEach((statement) => {
      if (visitor(statement, stackWithStatements)) {
        changed = true;
      }
      if (statement.type === 'IfStatement') {
        const ifStatement = statement as LogicScriptIfStatement;
        if (
          this.dfsStatementsInner(
            ifStatement.thenStatements as StatementType[],
            visitor,
            stackWithStatements,
          )
        ) {
          changed = true;
        }
        if (
          this.dfsStatementsInner(
            ifStatement.elseStatements as StatementType[],
            visitor,
            stackWithStatements,
          )
        ) {
          changed = true;
        }
      }
    });
    return changed;
  }

  findNextStatementPosition(
    statement: StatementType,
    stack: LogicScriptStatementStack<StatementType>,
  ): { index: number; stack: LogicScriptStatementStack<StatementType> } | undefined {
    const statementIndex = stack[0].indexOf(statement);
    if (statementIndex < stack[0].length - 1) {
      return { index: statementIndex + 1, stack };
    }

    if (stack.length > 1) {
      const enclosingStatement = stack[1].find(
        (s) =>
          s.type === 'IfStatement' &&
          ((s as LogicScriptIfStatement).thenStatements === stack[0] ||
            (s as LogicScriptIfStatement).elseStatements === stack[0]),
      );
      if (!enclosingStatement) {
        throw new Error(`Can't find enclosing statement`);
      }
      return this.findNextStatementPosition(enclosingStatement, stack.slice(1));
    }

    return undefined;
  }

  findNextStatement(
    statement: StatementType,
    stack: LogicScriptStatementStack<StatementType>,
  ): LogicScriptStatement | undefined {
    const position = this.findNextStatementPosition(statement, stack);
    if (!position) {
      return undefined;
    }

    return position.stack[0][position.index];
  }
}

export function parseLogicScript(source: string): LogicScriptParseTree<LogicScriptStatement> {
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
