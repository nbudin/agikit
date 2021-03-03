import assertNever from 'assert-never';
import { getGotoTargetLabel, LogicScriptParseTree } from '../../Scripting/LogicScriptParser';
import {
  generateLogicScript,
  generateLogicScriptForArgumentList,
  generateLogicScriptForBooleanExpression,
} from '../../Scripting/LogicScriptGenerator';
import {
  LogicScriptArgument,
  LogicScriptArithmeticAssignmentStatement,
  LogicScriptBooleanBinaryOperation,
  LogicScriptBooleanExpression,
  LogicScriptIdentifier,
  LogicScriptIfStatement,
  LogicScriptLiteral,
  LogicScriptStatement,
} from '../../Scripting/LogicScriptParserTypes';
import { AGICommandArgType } from '../../Types/AGICommands';
import {
  LogicResource,
  LogicConditionClause,
  LogicCommand,
  LogicInstruction,
} from '../../Types/Logic';
import { WordList } from '../../Types/WordList';
import { optimizeAST } from './ASTOptimization';
import {
  BasicBlock,
  BasicBlockEdge,
  BasicBlockGraph,
  IfExitBasicBlock,
  ReverseCFGNode,
  SinglePathBasicBlock,
} from './ControlFlowAnalysis';
import { DominatorTree } from './DominatorTree';
import { decompileInstructions, LogicLabel } from './LogicDecompile';
import { generateLabels } from './LogicDisasm';

export type CodeGenerationContext = {
  logic: LogicResource;
  wordList: WordList;
};

function generateWordArg(value: number, context: CodeGenerationContext): LogicScriptLiteral {
  const word = context.wordList.get(value);
  if (!word) {
    throw new Error(`Word ${value} not found`);
  }
  const canonicalWord = [...word.values()][0];
  return { type: 'Literal', value: canonicalWord };
}

function generateMessageArg(context: CodeGenerationContext, value: number): LogicScriptLiteral {
  const message = context.logic.messages[value - 1];
  if (!message) {
    throw new Error(`Message ${value} not found`);
  }
  return { type: 'Literal', value: message };
}

function generateArg(
  value: number,
  argType: AGICommandArgType,
  context: CodeGenerationContext,
): LogicScriptLiteral | LogicScriptIdentifier {
  switch (argType) {
    case AGICommandArgType.Number:
      return { type: 'Literal', value };
    case AGICommandArgType.Flag:
      return { type: 'Identifier', name: `f${value}` };
    case AGICommandArgType.CtrlCode:
      return { type: 'Identifier', name: `c${value}` };
    case AGICommandArgType.Item:
      return { type: 'Identifier', name: `i${value}` };
    case AGICommandArgType.Object:
      return { type: 'Identifier', name: `o${value}` };
    case AGICommandArgType.String:
      return { type: 'Identifier', name: `s${value}` };
    case AGICommandArgType.Variable:
      return { type: 'Identifier', name: `v${value}` };
    case AGICommandArgType.Word:
      return generateWordArg(value, context);
    case AGICommandArgType.Message:
      return generateMessageArg(context, value);
    default:
      assertNever(argType);
  }
}

function areArgumentsEqual(a: LogicScriptArgument, b: LogicScriptArgument): boolean {
  if (a.type === 'Identifier') {
    if (b.type !== 'Identifier') {
      return false;
    }

    return a.name === b.name;
  }

  if (b.type !== 'Literal') {
    return false;
  }

  return a.value === b.value;
}

function doOperationArgumentsMatch(
  a: LogicScriptBooleanBinaryOperation,
  b: LogicScriptBooleanBinaryOperation,
): boolean {
  if (areArgumentsEqual(a.left, b.left) && areArgumentsEqual(a.right, b.right)) {
    return true;
  }

  if (a.operator === '==' || a.operator === '!=' || b.operator === '==' || b.operator === '!=') {
    if (areArgumentsEqual(a.right, b.left) && areArgumentsEqual(a.left, b.right)) {
      return true;
    }
  }

  return false;
}

export function generateBooleanExpression(
  clauses: LogicConditionClause[],
  context: CodeGenerationContext,
): LogicScriptBooleanExpression {
  if (clauses.length > 1) {
    return {
      type: 'AndExpression',
      clauses: clauses.map((clause) => generateBooleanExpression([clause], context)),
    };
  }
  const clause = clauses[0];
  if (clause.type === 'or') {
    const clauses = clause.orTests.map((orTest) => generateBooleanExpression([orTest], context));

    // TODO: only do this if not in standards mode
    if (
      clauses.length === 2 &&
      clauses[0].type === 'BooleanBinaryOperation' &&
      clauses[1].type === 'BooleanBinaryOperation'
    ) {
      const [left, right] = clauses;

      if (
        left.operator === '<' &&
        right.operator === '==' &&
        doOperationArgumentsMatch(left, right)
      ) {
        return {
          type: 'BooleanBinaryOperation',
          operator: '<=',
          left: left.left,
          right: left.right,
        };
      }

      if (
        left.operator === '==' &&
        right.operator === '<' &&
        doOperationArgumentsMatch(left, right)
      ) {
        return {
          type: 'BooleanBinaryOperation',
          operator: '<=',
          left: right.left,
          right: right.right,
        };
      }

      if (
        left.operator === '>' &&
        right.operator === '==' &&
        doOperationArgumentsMatch(left, right)
      ) {
        return {
          type: 'BooleanBinaryOperation',
          operator: '>=',
          left: left.left,
          right: left.right,
        };
      }

      if (
        left.operator === '==' &&
        right.operator === '>' &&
        doOperationArgumentsMatch(left, right)
      ) {
        return {
          type: 'BooleanBinaryOperation',
          operator: '>=',
          left: right.left,
          right: right.right,
        };
      }
    }

    return { type: 'OrExpression', clauses };
  }

  const argumentList = clause.args.map((value, index) =>
    generateArg(
      value,
      clause.testCommand.varArgs ? AGICommandArgType.Word : clause.testCommand.argTypes[index],
      context,
    ),
  );

  if (clause.testCommand.name === 'equaln' || clause.testCommand.name === 'equalv') {
    return {
      type: 'BooleanBinaryOperation',
      operator: clause.negate ? '!=' : '==',
      left: argumentList[0],
      right: argumentList[1],
    };
  }

  if (clause.negate) {
    return {
      type: 'NotExpression',
      expression: generateBooleanExpression([{ ...clause, negate: false }], context),
    };
  }

  if (clause.testCommand.name === 'lessn' || clause.testCommand.name === 'lessv') {
    return {
      type: 'BooleanBinaryOperation',
      operator: '<',
      left: argumentList[0],
      right: argumentList[1],
    };
  }

  if (clause.testCommand.name === 'greatern' || clause.testCommand.name === 'greaterv') {
    return {
      type: 'BooleanBinaryOperation',
      operator: '>',
      left: argumentList[0],
      right: argumentList[1],
    };
  }

  return {
    type: 'TestCall',
    testName: clause.testCommand.name,
    argumentList,
  };
}

export function generateLogicCommandCode(
  instruction: LogicCommand,
  context: CodeGenerationContext,
): string {
  const argumentList = generateLogicScriptForArgumentList(
    instruction.args.map((arg, i) => generateArg(arg, instruction.agiCommand.argTypes[i], context)),
  );

  return `${instruction.agiCommand.name}(${argumentList});`;
}

function generateLogicAsmInstruction(
  instruction: LogicInstruction,
  labels: LogicLabel[],
  context: CodeGenerationContext,
) {
  if (instruction.type === 'command') {
    return generateLogicCommandCode(instruction, context);
  }

  if (instruction.type === 'goto') {
    const jumpLabel = labels.find((label) => label.address === instruction.jumpAddress);
    if (!jumpLabel) {
      throw new Error(`Unlabeled jump address: ${instruction.jumpAddress}`);
    }

    return `goto ${jumpLabel.label};`;
  }

  const skipLabel = labels.find((label) => label.address === instruction.skipAddress);
  if (!skipLabel) {
    throw new Error(`Unlabeled jump address: ${instruction.skipAddress}`);
  }

  const expression = generateLogicScriptForBooleanExpression(
    generateBooleanExpression(instruction.clauses, context),
  );
  return `unless (${expression}) goto ${skipLabel.label};`;
}

function generateLogicAsmInstructionWithPossibleLabel(
  instruction: LogicInstruction,
  labels: LogicLabel[],
  context: CodeGenerationContext,
) {
  const lineLabel = labels.find((label) => label.address === instruction.address);
  const lineInstruction = `${instruction.address} ${generateLogicAsmInstruction(
    instruction,
    labels,
    context,
  )}`;
  if (lineLabel) {
    return `\n${lineLabel.label}:\n${lineInstruction}`;
  }

  return lineInstruction;
}

export function generateLogicMessages(logic: LogicResource): string {
  const messages = logic.messages.map((message, index) =>
    message == null
      ? undefined
      : `#message ${index + 1} "${message
          ?.replace(/"/g, '\\"')
          .replace(/\n/g, '\\n')
          .replace(/\r/g, '\\r')}"`,
  );
  return `// messages\n${messages.filter((message) => message != null).join('\n')}\n`;
}

export function generateLogicAsm(
  logic: LogicResource,
  wordList: WordList,
  labels?: LogicLabel[],
): string {
  const labelsToUse = generateLabels(logic.instructions, labels);
  const asmCode = logic.instructions.map((instruction) =>
    generateLogicAsmInstructionWithPossibleLabel(instruction, labelsToUse, { logic, wordList }),
  );

  return `${asmCode.join('\n')}\n\n${generateLogicMessages(logic)}`;
}

export class LogicScriptGenerator {
  private graph: BasicBlockGraph;
  private context: CodeGenerationContext;
  private dominatorTree: DominatorTree<BasicBlock>;
  private postDominatorTree: DominatorTree<ReverseCFGNode>;
  private visited: Set<BasicBlock>;

  constructor(graph: BasicBlockGraph, context: CodeGenerationContext) {
    this.graph = graph;
    this.context = context;
    this.visited = new Set<BasicBlock>();
    this.dominatorTree = graph.buildDominatorTree();
    this.postDominatorTree = graph.buildPostDominatorTree();
  }

  generateCode(): string {
    const queue: BasicBlock[] = [this.graph.root];
    const statements = [];

    while (queue.length > 0) {
      const block = queue.shift();
      if (!block || this.visited.has(block)) {
        continue;
      }
      statements.push(...this.generateCodeForBasicBlock(block, queue));
    }

    const parseTree = new LogicScriptParseTree(statements);

    this.removeUnusedLabels(parseTree);
    this.removeRedundantJumps(parseTree);
    this.removeEmptyThenWithElse(parseTree);

    return generateLogicScript(parseTree.program);
  }

  private removeUnusedLabels(parseTree: LogicScriptParseTree<LogicScriptStatement>) {
    const usedLabels = new Set<string>();

    parseTree.dfsStatements((statement) => {
      const targetLabel = getGotoTargetLabel(statement);
      if (targetLabel) {
        usedLabels.add(targetLabel);
      }
      return false;
    });

    parseTree.dfsStatements((statement, stack) => {
      if (statement.type === 'Label' && !usedLabels.has(statement.label)) {
        const statementList = stack[0];
        statementList.splice(statementList.indexOf(statement), 1);
        return true;
      }
      return false;
    });
  }

  private removeRedundantJumps(parseTree: LogicScriptParseTree<LogicScriptStatement>) {
    parseTree.dfsStatements((statement, stack) => {
      const targetLabel = getGotoTargetLabel(statement);
      if (targetLabel) {
        const nextStatement = parseTree.findNextStatement(statement, stack);
        if (
          (nextStatement?.type === 'Label' && nextStatement.label === targetLabel) ||
          (nextStatement && getGotoTargetLabel(nextStatement) === targetLabel)
        ) {
          const statementList = stack[0];
          statementList.splice(statementList.indexOf(statement), 1);
          return true;
        }
      }

      return false;
    });
  }

  private removeEmptyThenWithElse(parseTree: LogicScriptParseTree<LogicScriptStatement>) {
    parseTree.dfsStatements((statement) => {
      if (
        statement.type === 'IfStatement' &&
        statement.thenStatements.length === 0 &&
        statement.elseStatements.length > 0
      ) {
        statement.conditions = {
          type: 'NotExpression',
          expression: statement.conditions,
        };
        statement.thenStatements = statement.elseStatements;
        statement.elseStatements = [];
        return true;
      }
      return false;
    });
  }

  private dominates(a: BasicBlock, b: BasicBlock): boolean {
    return this.dominatorTree.dominates(a.id, b.id);
  }

  private immediatelyDominates(a: BasicBlock, b: BasicBlock): boolean {
    return this.dominatorTree.immediatelyDominates(a.id, b.id);
  }

  private postDominates(a: BasicBlock, b: BasicBlock): boolean {
    return this.postDominatorTree.dominates(a.id, b.id);
  }

  private immediatelyPostDominates(a: BasicBlock, b: BasicBlock): boolean {
    return this.postDominatorTree.immediatelyDominates(a.id, b.id);
  }

  private generateCodeForBasicBlock(
    block: BasicBlock,
    queue: BasicBlock[],
  ): LogicScriptStatement[] {
    if (this.visited.has(block)) {
      const label = block.label;
      if (!label) {
        throw new Error('Jump to unlabeled statement');
      }
      return [this.generateGoto(label)];
    }

    this.visited.add(block);

    if (block.type === 'singlePathBasicBlock') {
      return this.generateSinglePathCode(block, queue);
    }

    if (block.type === 'ifExitBasicBlock') {
      return this.generateIfCode(block, queue);
    }

    return assertNever(block);
  }

  private generatePreamble(block: BasicBlock): LogicScriptStatement[] {
    const blockLabel = this.findBasicBlockLabel(block);
    const statements: LogicScriptStatement[] = [];

    if (blockLabel) {
      statements.push({ type: 'Label', label: blockLabel.label });
    }

    block.commands.forEach((command) => {
      const argumentList = command.args.map((value, index) => {
        const argumentType = command.agiCommand.argTypes[index];
        return generateArg(value, argumentType, this.context);
      });

      const commandName = command.agiCommand.name;
      if (
        (commandName === 'increment' || commandName === 'decrement') &&
        argumentList.length === 1 &&
        argumentList[0].type === 'Identifier'
      ) {
        statements.push({
          type: 'UnaryOperationStatement',
          identifier: argumentList[0],
          operation: commandName === 'increment' ? '++' : '--',
        });
      } else if (
        (commandName === 'assignn' || commandName === 'assignv') &&
        argumentList.length === 2 &&
        argumentList[0].type === 'Identifier'
      ) {
        statements.push({
          type: 'ValueAssignmentStatement',
          assignee: argumentList[0],
          value: argumentList[1],
        });
      } else if (
        (commandName === 'addn' ||
          commandName === 'addv' ||
          commandName === 'subn' ||
          commandName === 'subv' ||
          commandName === 'mul.n' ||
          commandName === 'mul.v' ||
          commandName === 'div.n' ||
          commandName === 'div.v') &&
        argumentList.length === 2 &&
        argumentList[0].type === 'Identifier'
      ) {
        let operator: LogicScriptArithmeticAssignmentStatement['operator'];
        switch (commandName) {
          case 'addn':
          case 'addv':
            operator = '+';
            break;
          case 'subn':
          case 'subv':
            operator = '-';
            break;
          case 'mul.n':
          case 'mul.v':
            operator = '*';
            break;
          case 'div.n':
          case 'div.v':
            operator = '/';
            break;
        }

        statements.push({
          type: 'ArithmeticAssignmentStatement',
          operator,
          assignee: argumentList[0],
          value: argumentList[1],
        });
      } else {
        statements.push({
          type: 'CommandCall',
          argumentList: argumentList,
          commandName: commandName,
        });
      }
    });

    return statements;
  }

  private generateSinglePathCode(
    block: SinglePathBasicBlock,
    queue: BasicBlock[],
  ): LogicScriptStatement[] {
    const preamble = this.generatePreamble(block);
    if (block.next) {
      if (this.dominates(block, block.next.to) && this.postDominates(block.next.to, block)) {
        const nextBlockCode = this.generateCodeForBasicBlock(block.next.to, queue);
        return [...preamble, ...nextBlockCode];
      }

      const nextBlockLabel = this.findBasicBlockLabel(block.next.to);
      if (nextBlockLabel) {
        queue.push(block.next.to);
        if (this.visited.has(block.next.to)) {
          return [...preamble, this.generateGoto(nextBlockLabel)];
        }
      }
    }

    return preamble;
  }

  private generateGoto(label: LogicLabel): LogicScriptStatement {
    return {
      type: 'CommandCall',
      commandName: 'goto',
      argumentList: [{ type: 'Identifier', name: label.label }],
    };
  }

  private generateIfCode(block: IfExitBasicBlock, queue: BasicBlock[]): LogicScriptStatement[] {
    const [thenStatements, thenQueue] = this.generateBranchCode(block, block.then);
    const [elseStatements, elseQueue] = this.generateBranchCode(block, block.else);

    const ifStatement: LogicScriptIfStatement = {
      type: 'IfStatement',
      conditions: generateBooleanExpression(block.clauses, this.context),
      thenStatements,
      elseStatements,
    };

    const subsequentCode = [];
    const elseBlock = block.else;

    if (
      // elseQueue.length === 0 &&
      elseBlock &&
      this.postDominates(elseBlock.to, block) &&
      thenQueue.every((thenBlock) => this.dominates(elseBlock.to, thenBlock))
    ) {
      // else clause can be unrolled
      ifStatement.elseStatements = [];
      subsequentCode.push(...elseStatements);
      thenQueue.splice(0, thenQueue.length);
    }

    const innerQueue = [...thenQueue, ...elseQueue];
    while (innerQueue.length > 0) {
      const innerBlock = innerQueue.shift();
      if (!innerBlock || this.visited.has(innerBlock)) {
        continue;
      }
      if (!this.dominates(block, innerBlock)) {
        queue.push(innerBlock);
        continue;
      }
      subsequentCode.push(...this.generateCodeForBasicBlock(innerBlock, innerQueue));
    }
    return [...this.generatePreamble(block), ifStatement, ...subsequentCode];
  }

  private generateBranchCode(
    block: IfExitBasicBlock,
    branch?: BasicBlockEdge,
  ): [LogicScriptStatement[], BasicBlock[]] {
    if (branch && !this.dominates(block, branch.to)) {
      const label = this.findBasicBlockLabel(branch.to);
      if (!label) {
        throw new Error(`Can't generate goto for unlabeled block ${branch.to.id}`);
      }
      return [[this.generateGoto(label)], [branch.to]];
    }
    const branchQueue: BasicBlock[] = [];
    const branchCode = branch ? this.generateCodeForBasicBlock(branch.to, branchQueue) : [];
    if (branch && branchQueue.length > 0 && !this.postDominates(branchQueue[0], branch.to)) {
      const label = this.findBasicBlockLabel(branchQueue[0]);
      if (!label) {
        throw new Error(`Can't generate goto for unlabeled block ${branchQueue[0].id}`);
      }
      branchCode.push(this.generateGoto(label));
    }
    return [branchCode, branchQueue];
  }

  private findBasicBlockLabel(block: BasicBlock): LogicLabel | undefined {
    if (block.commands.length > 0) {
      return block.commands[0].label;
    }

    if (block.type === 'singlePathBasicBlock') {
      return block.metadata.gotoNode?.label;
    }

    if (block.type === 'ifExitBasicBlock') {
      return block.metadata.ifNode.label;
    }

    assertNever(block);
  }
}

export function generateCodeForLogicResource(
  logic: LogicResource,
  wordList: WordList,
): [string, BasicBlockGraph] {
  const root = decompileInstructions(logic.instructions);
  const optimizedRoot = optimizeAST(root);
  const scriptGenerator = new LogicScriptGenerator(optimizedRoot, { logic, wordList });
  return [scriptGenerator.generateCode() + '\n\n' + generateLogicMessages(logic), optimizedRoot];
}
