import { AGICommandArgType, agiCommandsByName, testCommandsByName } from '../Types/AGICommands';
import {
  LogicASTNode,
  LogicCommandNode,
  LogicGotoNode,
  LogicIfNode,
  LogicLabel,
} from '../Extract/Logic/LogicDecompile';
import {
  LogicScriptParseTree,
  LogicScriptPreprocessedStatement,
  LogicScriptStatementStack,
} from './LogicScriptParser';
import {
  LogicScriptIdentifier,
  LogicScriptLabel,
  LogicScriptLiteral,
  LogicScriptTestCall,
} from './LogicScriptParserTypes';
import { WordList } from '../Types/WordList';
import { flatMap, max } from 'lodash';
import assertNever from 'assert-never';
import { LogicConditionClause, LogicTest } from '../Types/Logic';
import { simplifyLogicScriptExpression, StrictBooleanExpression } from './PropositionalLogic';
import {
  LogicScriptPrimitiveStatement,
  simplifyLogicScriptProgram,
} from './LogicScriptPrimitiveTree';
import { IdentifierMapping } from './LogicScriptIdentifierMapping';
import { ObjectList, ObjectListEntry } from '../Types/ObjectList';

const fakeJumpTarget: LogicCommandNode = {
  type: 'command',
  id: 'fakeJumpTarget',
  address: -1,
  agiCommand: agiCommandsByName.return,
  args: [],
};

export class LogicScriptASTGenerator {
  parseTree: LogicScriptParseTree<LogicScriptPrimitiveStatement>;
  wordList: WordList;
  invertedWordList: Map<string, number>;
  objectList: ObjectList;
  objectNumbersByName: Map<string, number>;
  messages: (string | undefined)[];
  messagesByContent: Map<string, number>;
  identifiers: Map<string, IdentifierMapping>;
  unresolvedGotos: { node: LogicGotoNode; label: string }[];
  labels: Map<string, LogicLabel>;
  private statementAddresses: Map<LogicScriptPrimitiveStatement, number>;
  private nodesByAddress: Map<number, LogicASTNode>;

  constructor(
    parseTree: LogicScriptParseTree<LogicScriptPreprocessedStatement>,
    wordList: WordList,
    objectList: ObjectList,
  ) {
    this.parseTree = new LogicScriptParseTree(
      simplifyLogicScriptProgram(parseTree.program),
      parseTree.identifiers,
    );
    this.wordList = wordList;
    this.objectList = objectList;

    this.invertedWordList = new Map<string, number>(
      flatMap([...this.wordList.entries()], ([wordNumber, words]) =>
        [...words].map((word) => [word, wordNumber]),
      ),
    );
    this.objectNumbersByName = new Map(
      this.objectList.objects.map((object, index) => [object.name, index]),
    );

    this.unresolvedGotos = [];
    this.labels = new Map<string, LogicLabel>();
    this.identifiers = this.parseTree.identifiers;

    this.statementAddresses = new Map<LogicScriptPrimitiveStatement, number>();
    this.nodesByAddress = new Map<number, LogicASTNode>();
    this.messages = [];
    this.messagesByContent = new Map<string, number>();
    let address = 1;
    this.parseTree.dfsStatements((statement) => {
      this.statementAddresses.set(statement, address);
      address += 10;

      if (statement.type === 'MessageDirective') {
        this.messages[statement.number - 1] = statement.message;
        if (!this.messagesByContent.has(statement.message)) {
          this.messagesByContent.set(statement.message, statement.number);
        }
      }

      return false;
    });
  }

  private getMessageNumber(message: string): number {
    const messageNumber = this.messagesByContent.get(message);
    if (messageNumber != null) {
      return messageNumber;
    }
    const newMessageNumber = (max([...this.messagesByContent.values()]) ?? 0) + 1;
    this.messagesByContent.set(message, newMessageNumber);
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

      if (argumentType === AGICommandArgType.Message) {
        return this.getMessageNumber(argument.value);
      }

      if (argumentType === AGICommandArgType.Word) {
        const wordNumber = this.invertedWordList.get(argument.value);
        if (wordNumber == null) {
          throw new Error(`Word ${argument.value} not found in word list`);
        }
        return wordNumber;
      }

      if (argumentType === AGICommandArgType.Item) {
        const objectNumber = this.objectNumbersByName.get(argument.value);
        if (objectNumber == null) {
          throw new Error(`Item "${argument.value}" not found in object list`);
        }
        return objectNumber;
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

      if (identifierMapping.identifierType === 'constant') {
        if (typeof identifierMapping.value === 'number') {
          return identifierMapping.value;
        } else {
          return this.getMessageNumber(identifierMapping.value);
        }
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
      args: expression.argumentList.map((argument, index) => {
        const argumentType =
          testCommand.name === 'said' ? AGICommandArgType.Word : testCommand.argTypes[index];
        return this.argumentToNumber(argument, argumentType);
      }),
      negate: false,
    };
  }

  private generateASTForNextStatement(
    statement: LogicScriptPrimitiveStatement,
    stack: LogicScriptStatementStack<LogicScriptPrimitiveStatement>,
  ) {
    const nextStatementPosition = this.parseTree.findNextStatementPosition(statement, stack);
    if (!nextStatementPosition) {
      return undefined;
    }
    return this.generateASTForLogicScriptStatements(
      nextStatementPosition.stack[0].slice(nextStatementPosition.index),
      undefined,
      nextStatementPosition.stack,
    );
  }

  generateASTForLogicScriptStatements(
    statements: LogicScriptPrimitiveStatement[],
    previousLabel: LogicScriptLabel | undefined,
    stack: LogicScriptStatementStack<LogicScriptPrimitiveStatement>,
  ): LogicASTNode | undefined {
    const statement = statements[0];
    const address = this.statementAddresses.get(statement);
    if (!address) {
      throw new Error('Address not found');
    }

    const existingNode = this.nodesByAddress.get(address);
    if (existingNode) {
      return existingNode;
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

      const node: LogicCommandNode = {
        type: 'command',
        address,
        id: address.toString(10),
        agiCommand,
        label,
        args: statement.argumentList.map((argument, index) =>
          this.argumentToNumber(argument, agiCommand.argTypes[index]),
        ),
        next: this.generateASTForNextStatement(statement, stack),
      };
      this.nodesByAddress.set(address, node);
      return node;
    }

    if (statement.type === 'IfStatement') {
      const clauses: LogicConditionClause[] = this.booleanExpressionToClauses(
        simplifyLogicScriptExpression(statement.conditions, this.identifiers),
      );
      const node: LogicIfNode = {
        type: 'if',
        id: address.toString(10),
        label,
        clauses,
        then:
          statement.thenStatements.length > 0
            ? this.generateASTForLogicScriptStatements(statement.thenStatements, undefined, [
                statement.thenStatements,
                ...stack,
              ])
            : this.generateASTForNextStatement(statement, stack),
        else:
          statement.elseStatements.length > 0
            ? this.generateASTForLogicScriptStatements(statement.elseStatements, undefined, [
                statement.elseStatements,
                ...stack,
              ])
            : this.generateASTForNextStatement(statement, stack),
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

  generateASTForLogicScript(): LogicASTNode {
    const root = this.generateASTForLogicScriptStatements(this.parseTree.program, undefined, [
      this.parseTree.program,
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

  getLabels(): LogicLabel[] {
    return [...this.labels.values()];
  }

  generateMessageArray(): (string | undefined)[] {
    return this.messages;
  }
}
