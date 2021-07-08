import {
  LogicScriptDefineDirective,
  LogicScriptIdentifier,
  LogicScriptIfStatement,
  LogicScriptIncludeDirective,
  LogicScriptProgram,
  LogicScriptStatement,
} from './LogicScriptParserTypes';
import { parse, SyntaxError } from './LogicScriptParser.generated';
import { flatMap } from 'lodash';
import path from 'path';
import fs from 'fs';
import {
  BUILT_IN_IDENTIFIERS,
  IdentifierMapping,
  resolveIdentifierMapping,
} from './LogicScriptIdentifierMapping';

export class SyntaxErrorWithFilePath extends SyntaxError {
  filePath: string;
}

export type LogicScriptStatementVisitor<StatementType extends LogicScriptStatement> = (
  statement: StatementType,
  stack: LogicScriptStatementStack<StatementType>,
) => boolean;

export type LogicScriptStatementStack<
  StatementType extends LogicScriptStatement
> = StatementType[][];

export type LogicScriptPreprocessedStatement =
  | Exclude<
      LogicScriptStatement,
      LogicScriptIncludeDirective | LogicScriptDefineDirective | LogicScriptIfStatement
    >
  | LogicScriptIfStatement<LogicScriptPreprocessedStatement>;

export function getGotoTargetLabel(statement: LogicScriptStatement): string | undefined {
  if (statement.type === 'CommandCall' && statement.commandName === 'goto') {
    const labelIdentifier = statement.argumentList[0] as LogicScriptIdentifier;
    return labelIdentifier.name;
  }
  return undefined;
}

export class LogicScriptParseTree<StatementType extends LogicScriptStatement> {
  program: LogicScriptProgram<StatementType>;
  identifiers: Map<string, IdentifierMapping>;

  constructor(
    program: LogicScriptProgram<StatementType>,
    identifiers: Map<string, IdentifierMapping>,
  ) {
    this.program = program;
    this.identifiers = identifiers;
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
    for (
      let nextStatementIndex = statementIndex + 1;
      nextStatementIndex < stack[0].length;
      nextStatementIndex++
    ) {
      const possibleNextStatement = stack[0][nextStatementIndex];
      if (
        // we actually do want to consider labels as statements since LogicScriptASTGenerator relies
        // on running into them
        possibleNextStatement.type !== 'Comment' &&
        possibleNextStatement.type !== 'DefineDirective' &&
        possibleNextStatement.type !== 'IncludeDirective' &&
        possibleNextStatement.type !== 'MessageDirective'
      ) {
        return { index: nextStatementIndex, stack };
      }
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
  ): StatementType | undefined {
    const position = this.findNextStatementPosition(statement, stack);
    if (!position) {
      return undefined;
    }

    return position.stack[0][position.index];
  }
}

type PreprocessorOutput = LogicScriptProgram<LogicScriptPreprocessedStatement>;

export function buildIdentifierMappingForDefineDirective(
  directive: LogicScriptDefineDirective,
  identifierMappings: Map<string, IdentifierMapping>,
): IdentifierMapping {
  if (directive.value.type === 'Identifier') {
    return {
      ...resolveIdentifierMapping(directive.value.name, identifierMappings),
      name: directive.identifier.name,
    };
  } else {
    return {
      identifierType: 'constant',
      name: directive.identifier.name,
      value: directive.value.value,
    };
  }
}

function preprocessStatement(
  statement: LogicScriptStatement,
  scriptPath: string,
  identifierMappings: Map<string, IdentifierMapping>,
): PreprocessorOutput {
  if (statement.type === 'IncludeDirective') {
    const includePath = path.resolve(path.dirname(scriptPath), statement.filename.value);
    const includeSource = fs.readFileSync(includePath, 'utf-8');
    const includeProgram = parseLogicScriptRaw(includeSource, includePath);
    return preParseLogicScript(includeProgram, includePath, identifierMappings);
  }

  if (statement.type === 'DefineDirective') {
    if (identifierMappings.has(statement.identifier.name)) {
      throw new Error(`Identifier ${statement.identifier.name} is #defined multiple times`);
    }

    const mapping = buildIdentifierMappingForDefineDirective(statement, identifierMappings);
    identifierMappings.set(statement.identifier.name, mapping);

    return [];
  }

  if (statement.type === 'IfStatement') {
    const preprocessClause = (statements: LogicScriptStatement[]) => {
      return flatMap(statements, (s) => preprocessStatement(s, scriptPath, identifierMappings));
    };
    return [
      {
        type: 'IfStatement',
        conditions: statement.conditions,
        thenStatements: preprocessClause(statement.thenStatements),
        elseStatements: preprocessClause(statement.elseStatements),
        location: statement.location,
        ifKeyword: statement.ifKeyword,
        elseKeyword: statement.elseKeyword,
      },
    ];
  }

  return [statement];
}

export function parseLogicScriptRaw(
  source: string,
  scriptPath: string,
): LogicScriptProgram<LogicScriptStatement> {
  let rawProgram: LogicScriptProgram<LogicScriptStatement>;
  try {
    rawProgram = parse(source);
  } catch (error) {
    if (error.name === 'SyntaxError' && !('filePath' in error)) {
      const errorWithPath = new SyntaxErrorWithFilePath(
        error.message,
        error.expected,
        error.found,
        error.location,
      );
      errorWithPath.filePath = scriptPath;
      throw errorWithPath;
    }

    throw error;
  }
  return rawProgram;
}

function preParseLogicScript(
  rawProgram: LogicScriptProgram<LogicScriptStatement>,
  scriptPath: string,
  identifierMappings: Map<string, IdentifierMapping>,
): PreprocessorOutput {
  const preprocessedProgram: LogicScriptProgram<LogicScriptPreprocessedStatement> = flatMap(
    rawProgram,
    (statement) => preprocessStatement(statement, scriptPath, identifierMappings),
  );

  return preprocessedProgram;
}

export function parseLogicScript(
  rawProgram: LogicScriptProgram<LogicScriptStatement>,
  scriptPath: string,
): LogicScriptParseTree<LogicScriptPreprocessedStatement> {
  const identifierMappings = new Map<string, IdentifierMapping>(BUILT_IN_IDENTIFIERS);
  const program = preParseLogicScript(rawProgram, scriptPath, identifierMappings);
  const parseTree = new LogicScriptParseTree(program, identifierMappings);
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
