import assertNever from 'assert-never';
import {
  generateLogicScript,
  generateLogicScriptForArgumentList,
  generateLogicScriptForBooleanExpression,
} from '../../Scripting/LogicScriptGenerator';
import {
  LogicScriptBooleanExpression,
  LogicScriptIdentifier,
  LogicScriptIfStatement,
  LogicScriptLiteral,
  LogicScriptProgram,
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
    return {
      type: 'OrExpression',
      clauses: clause.orTests.map((orTest) => generateBooleanExpression([orTest], context)),
    };
  }

  if (clause.negate) {
    return {
      type: 'NotExpression',
      expression: generateBooleanExpression([{ ...clause, negate: false }], context),
    };
  }

  const argumentList = clause.args.map((value, index) =>
    generateArg(
      value,
      clause.testCommand.varArgs ? AGICommandArgType.Word : clause.testCommand.argTypes[index],
      context,
    ),
  );

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
  const messages = logic.messages.map((message, index) => `#message ${index + 1} "${message}"`);
  return `// messages\n${messages.join('\n')}\n`;
}

export function generateLogicAsm(logic: LogicResource, wordList: WordList): string {
  const labels = generateLabels(logic.instructions);
  const asmCode = logic.instructions.map((instruction) =>
    generateLogicAsmInstructionWithPossibleLabel(instruction, labels, { logic, wordList }),
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

    this.removeUnusedLabels(statements);

    // const jumpLabels = [...code.matchAll(/\bgoto\((\w+)\);\n/g)].map((match) => match[1]);
    // const codeWithUnusedJumpsRemoved = code.replace(/ *(\w+):\n/g, (labelLine, label) => {
    //   if (jumpLabels.includes(label)) {
    //     return labelLine;
    //   }
    //   return '';
    // });

    // let codeWithRedundantJumpsRemoved = codeWithUnusedJumpsRemoved;
    // let removedRedundantJumps = false;
    // do {
    //   removedRedundantJumps = false;
    //   codeWithRedundantJumpsRemoved = codeWithRedundantJumpsRemoved.replace(
    //     /\bgoto\((\w+)\);([\s}]*\1:)/gm,
    //     (match, label, afterLabel) => {
    //       removedRedundantJumps = true;
    //       return afterLabel;
    //     },
    //   );
    // } while (removedRedundantJumps);
    // return codeWithRedundantJumpsRemoved;

    return generateLogicScript(statements);
  }

  private removeUnusedLabels(program: LogicScriptProgram) {
    const usedLabels = new Set<string>();

    this.dfsStatements(program, (statement) => {
      if (statement.type === 'CommandCall' && statement.commandName === 'goto') {
        const labelIdentifier = statement.argumentList[0] as LogicScriptIdentifier;
        usedLabels.add(labelIdentifier.name);
      }
      return false;
    });

    this.dfsStatements(program, (statement, statementList) => {
      if (statement.type === 'Label' && !usedLabels.has(statement.label)) {
        statementList.splice(statementList.indexOf(statement), 1);
        return true;
      }
      return false;
    });
  }

  private dfsStatements(
    statements: LogicScriptStatement[],
    visitor: (statement: LogicScriptStatement, statementList: LogicScriptStatement[]) => boolean,
  ): boolean {
    let changed = false;
    [...statements].forEach((statement) => {
      if (visitor(statement, statements)) {
        changed = true;
      }
      if (statement.type === 'IfStatement') {
        if (this.dfsStatements(statement.thenStatements, visitor)) {
          changed = true;
        }
        if (this.dfsStatements(statement.elseStatements, visitor)) {
          changed = true;
        }
      }
    });
    return changed;
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
      return [{ type: 'Comment', comment: ' WARNING: loop detected' }];
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
      statements.push({
        type: 'CommandCall',
        argumentList: command.args.map((value, index) => {
          const argumentType = command.agiCommand.argTypes[index];
          return generateArg(value, argumentType, this.context);
        }),
        commandName: command.agiCommand.name,
      });
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
