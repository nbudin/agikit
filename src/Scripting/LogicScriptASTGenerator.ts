import { AGICommandArgType, agiCommandsByName, testCommandsByName } from '../Types/AGICommands';
import {
  LogicASTNode,
  LogicCommandNode,
  LogicGotoNode,
  LogicIfNode,
  LogicLabel,
} from '../Extract/Logic/LogicDecompile';
import { LogicScriptParseTree, LogicScriptStatementStack } from './LogicScriptParser';
import {
  LogicScriptIdentifier,
  LogicScriptLabel,
  LogicScriptLiteral,
  LogicScriptStatement,
  LogicScriptTestCall,
} from './LogicScriptParserTypes';
import { WordList } from '../Types/WordList';
import { flatMap, max } from 'lodash';
import assertNever from 'assert-never';
import { LogicConditionClause, LogicTest } from '../Types/Logic';
import { simplifyLogicScriptExpression, StrictBooleanExpression } from './PropositionalLogic';

export type IdentifierMapping = { name: string; number: number; type: AGICommandArgType };

const BUILT_IN_IDENTIFIERS = new Map<string, IdentifierMapping>(
  flatMap([...Array(256).keys()], (index) => [
    { name: `v${index}`, number: index, type: AGICommandArgType.Variable },
    { name: `f${index}`, number: index, type: AGICommandArgType.Flag },
    { name: `o${index}`, number: index, type: AGICommandArgType.Object },
    { name: `c${index}`, number: index, type: AGICommandArgType.CtrlCode },
    { name: `i${index}`, number: index, type: AGICommandArgType.Item },
    { name: `s${index}`, number: index, type: AGICommandArgType.String },
  ]).map((identifierMapping) => [identifierMapping.name, identifierMapping]),
);

const fakeJumpTarget: LogicCommandNode = {
  type: 'command',
  id: 'fakeJumpTarget',
  address: -1,
  agiCommand: agiCommandsByName.return,
  args: [],
};

export class LogicScriptASTGenerator {
  parseTree: LogicScriptParseTree;
  wordList: WordList;
  invertedWordList: Map<string, number>;
  messages: Map<string, number>;
  identifiers: Map<string, IdentifierMapping>;
  unresolvedGotos: { node: LogicGotoNode; label: string }[];
  labels: Map<string, LogicLabel>;
  private statementAddresses: Map<LogicScriptStatement, number>;
  private nodesByAddress: Map<number, LogicASTNode>;

  constructor(parseTree: LogicScriptParseTree, wordList: WordList) {
    this.parseTree = parseTree;
    this.wordList = wordList;

    this.invertedWordList = new Map<string, number>(
      flatMap([...this.wordList.entries()], ([wordNumber, words]) =>
        [...words].map((word) => [word, wordNumber]),
      ),
    );

    this.unresolvedGotos = [];
    this.labels = new Map<string, LogicLabel>();
    this.identifiers = new Map<string, IdentifierMapping>(BUILT_IN_IDENTIFIERS);

    this.statementAddresses = new Map<LogicScriptStatement, number>();
    this.nodesByAddress = new Map<number, LogicASTNode>();
    this.messages = new Map<string, number>();
    let address = 1;
    parseTree.dfsStatements((statement) => {
      this.statementAddresses.set(statement, address);
      address += 1;

      if (statement.type === 'MessageDirective') {
        this.messages.set(statement.message, statement.number);
      }

      return false;
    });
  }

  private getMessageNumber(message: string): number {
    const messageNumber = this.messages.get(message);
    if (messageNumber != null) {
      return messageNumber;
    }
    const newMessageNumber = (max([...this.messages.values()]) ?? 0) + 1;
    this.messages.set(message, newMessageNumber);
    return newMessageNumber;
  }

  private argumentToNumber(
    argument: LogicScriptLiteral | LogicScriptIdentifier,
    argumentType: AGICommandArgType,
  ): number {
    if (argument.type === 'Literal') {
      if (typeof argument.value === 'number') {
        return argument.value;
      }

      if (argumentType === 'Message') {
        return this.getMessageNumber(argument.value);
      }

      if (argumentType === 'Word') {
        const wordNumber = this.invertedWordList.get(argument.value);
        if (wordNumber == null) {
          throw new Error(`Word ${argument.value} not found in word list`);
        }
        return wordNumber;
      }

      throw new Error(
        `Invalid value ${JSON.stringify(argument.value)} for ${argumentType} argument`,
      );
    }

    if (argument.type === 'Identifier') {
      const identifierMapping = this.identifiers.get(argument.name);
      if (!identifierMapping) {
        throw new Error(`Unknown identifier ${argument.name}`);
      }

      if (identifierMapping.type !== argumentType) {
        throw new Error(
          `Type mismatch: expected ${argumentType} but ${argument.name} is a ${identifierMapping.type}`,
        );
      }

      return identifierMapping.number;
    }

    assertNever(argument);
  }

  private booleanExpressionToClauses(expression: StrictBooleanExpression): LogicConditionClause[] {
    if (expression.type === 'TestCall') {
      return [this.testCallToClause(expression)];
    }

    if (expression.type === 'StrictAndExpression') {
      return flatMap(expression.clauses, (clause) => this.booleanExpressionToClauses(clause));
    }

    if (expression.type === 'StrictOrExpression') {
      return [
        {
          type: 'or',
          orTests: expression.clauses.map((subClause) => {
            if (subClause.type === 'StrictNotExpression') {
              return {
                ...this.testCallToClause(subClause.expression),
                negate: true,
              };
            }

            return this.testCallToClause(subClause);
          }),
        },
      ];
    }

    if (expression.type === 'StrictNotExpression') {
      return [
        {
          ...this.testCallToClause(expression.expression),
          negate: true,
        },
      ];
    }

    assertNever(expression);
  }

  private testCallToClause(expression: LogicScriptTestCall): LogicTest {
    const testCommand = testCommandsByName[expression.testName];
    if (!testCommand) {
      throw new Error(`Unknown test command ${expression.testName}`);
    }

    return {
      type: 'test',
      testCommand,
      args: expression.argumentList.map((argument, index) =>
        this.argumentToNumber(argument, testCommand.argTypes[index]),
      ),
      negate: false,
    };
  }

  generateASTForLogicScriptStatements(
    statements: LogicScriptStatement[],
    previousLabel: LogicScriptLabel | undefined,
    stack: LogicScriptStatementStack,
  ): LogicASTNode | undefined {
    const statement = statements[0];
    const address = this.statementAddresses.get(statement);
    if (!address) {
      throw new Error('Address not found');
    }

    const label: LogicLabel | undefined = previousLabel
      ? {
          label: previousLabel.label,
          address,
          references: [],
        }
      : undefined;
    if (label) {
      this.labels.set(label.label, label);
    }

    if (statement.type === 'CommandCall') {
      if (statement.commandName === 'goto') {
        const targetArgument = statement.argumentList[0];
        if (targetArgument.type !== 'Identifier') {
          throw new Error(`Invalid argument for goto: ${JSON.stringify(targetArgument)}`);
        }
        const node: LogicGotoNode = {
          type: 'goto',
          id: address.toString(10),
          jumpTarget: fakeJumpTarget,
        };

        this.unresolvedGotos.push({ node, label: targetArgument.name });
        this.nodesByAddress.set(address, node);
        return node;
      }

      const agiCommand = agiCommandsByName[statement.commandName];

      if (!agiCommand) {
        throw new Error(`Unknown command ${statement.commandName}`);
      }

      const nextStatementPosition = this.parseTree.findNextStatementPosition(statement, stack);

      const node: LogicCommandNode = {
        type: 'command',
        address,
        id: address.toString(10),
        agiCommand,
        label,
        args: statement.argumentList.map((argument, index) =>
          this.argumentToNumber(argument, agiCommand.argTypes[index]),
        ),
        next: nextStatementPosition
          ? this.generateASTForLogicScriptStatements(
              nextStatementPosition.stack[0].slice(nextStatementPosition.index),
              undefined,
              nextStatementPosition.stack,
            )
          : undefined,
      };
      this.nodesByAddress.set(address, node);
      return node;
    }

    if (statement.type === 'IfStatement') {
      const clauses: LogicConditionClause[] = this.booleanExpressionToClauses(
        simplifyLogicScriptExpression(statement.conditions),
      );
      const node: LogicIfNode = {
        type: 'if',
        id: address.toString(10),
        label,
        clauses,
        then: this.generateASTForLogicScriptStatements(statement.thenStatements, undefined, [
          statement.thenStatements,
          ...stack,
        ]),
        else:
          statement.elseStatements.length > 0
            ? this.generateASTForLogicScriptStatements(statement.elseStatements, undefined, [
                statement.elseStatements,
                ...stack,
              ])
            : undefined,
      };
      this.nodesByAddress.set(address, node);
      return node;
    }

    if (
      statement.type === 'Comment' ||
      statement.type === 'Label' ||
      statement.type === 'MessageDirective'
    ) {
      if (statements.length === 1) {
        return undefined;
      }
      return this.generateASTForLogicScriptStatements(
        statements.slice(1),
        statement.type === 'Label' ? statement : previousLabel,
        stack,
      );
    }

    assertNever(statement);
  }

  generateASTForLogicScript(parseTree: LogicScriptParseTree): LogicASTNode {
    const root = this.generateASTForLogicScriptStatements(parseTree.program, undefined, [
      parseTree.program,
    ]);
    if (root == null) {
      throw new Error('Empty script');
    }

    this.unresolvedGotos.forEach((unresolvedGoto) => {
      const label = this.labels.get(unresolvedGoto.label);
      if (label == null) {
        throw new Error(`Invalid label: ${unresolvedGoto.label}`);
      }
      const target = this.nodesByAddress.get(label.address);
      if (target == null) {
        throw new Error(`No statement at label ${unresolvedGoto.label}`);
      }
      unresolvedGoto.node.jumpTarget = target;
    });

    return root;
  }
}
