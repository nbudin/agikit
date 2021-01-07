import assertNever from 'assert-never';
import { AGICommandArgType } from '../Types/AGICommands';
import {
  LogicResource,
  LogicConditionClause,
  LogicCommand,
  LogicInstruction,
  LogicLabel,
} from '../Types/Logic';
import { WordList } from '../Types/WordList';
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
import { decompileInstructions } from './LogicDecompile';
import { generateLabels } from './LogicDisasm';

export type CodeGenerationContext = {
  logic: LogicResource;
  wordList: WordList;
};

function generateWordArg(value: number, context: CodeGenerationContext) {
  const word = context.wordList.get(value);
  if (!word) {
    throw new Error(`Word ${value} not found`);
  }
  const canonicalWord = [...word.values()][0];
  return `"${canonicalWord}"`;
  2;
}

function generateArg(
  value: number,
  argType: AGICommandArgType,
  context: CodeGenerationContext,
): string {
  switch (argType) {
    case AGICommandArgType.Number:
      return value.toString(10);
    case AGICommandArgType.Flag:
      return `f${value}`;
    case AGICommandArgType.CtrlCode:
      return `c${value}`;
    case AGICommandArgType.Item:
      return `i${value}`;
    case AGICommandArgType.Object:
      return `o${value}`;
    case AGICommandArgType.String:
      return `s${value}`;
    case AGICommandArgType.Variable:
      return `v${value}`;
    case AGICommandArgType.Word:
      return generateWordArg(value, context);
    case AGICommandArgType.Message:
      return `"${context.logic.messages[value - 1]}"`;
    default:
      assertNever(argType);
  }
}

export function generateConditionClause(
  clause: LogicConditionClause,
  context: CodeGenerationContext,
): string {
  if (clause.type === 'or') {
    return clause.orTests.map((orTest) => generateConditionClause(orTest, context)).join(' || ');
  }

  const args = clause.args.map((value, index) =>
    generateArg(
      value,
      clause.testCommand.varArgs ? AGICommandArgType.Word : clause.testCommand.argTypes[index],
      context,
    ),
  );
  return `${clause.negate ? '!' : ''}${clause.testCommand.name}(${args.join(', ')})`;
}

export function generateLogicCommandCode(
  instruction: LogicCommand,
  context: CodeGenerationContext,
): string {
  return `${instruction.agiCommand.name}(${instruction.args
    .map((arg, i) => generateArg(arg, instruction.agiCommand.argTypes[i], context))
    .join(', ')});`;
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

  return `unless (${instruction.clauses
    .map((clause) => generateConditionClause(clause, context))
    .join(' && ')}) goto ${skipLabel.label};`;
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
    let code = '';

    while (queue.length > 0) {
      const block = queue.shift();
      if (!block || this.visited.has(block)) {
        continue;
      }
      code += this.generateCodeForBasicBlock(block, 0, queue);
    }

    const jumpLabels = [...code.matchAll(/\bgoto\((\w+)\);\n/g)].map((match) => match[1]);
    return code.replace(/ *(\w+):\n/g, (labelLine, label) => {
      if (jumpLabels.includes(label)) {
        return labelLine;
      }
      return '';
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
    indent: number,
    queue: BasicBlock[],
  ): string {
    if (this.visited.has(block)) {
      return '// WARNING: loop detected\n';
    }

    this.visited.add(block);

    if (block.type === 'singlePathBasicBlock') {
      console.log(`Single path ${block.id}`);
      return this.generateSinglePathCode(block, indent, queue);
    }

    if (block.type === 'ifExitBasicBlock') {
      console.log(`If block ${block.id}`);
      return this.generateIfCode(block, indent, queue);
    }

    return assertNever(block);
  }

  private generatePreamble(block: BasicBlock, indent: number) {
    const indentSpaces = ' '.repeat(indent);
    const blockLabel = this.findBasicBlockLabel(block);
    const labelIfPresent = blockLabel
      ? `${' '.repeat(indent < 2 ? indent : indent - 2)}${blockLabel.label}:\n`
      : '';

    const commandSection = block.commands
      .map((command) => `${indentSpaces}${generateLogicCommandCode(command, this.context)}`)
      .join('\n');
    const preamble = labelIfPresent + commandSection + (block.commands.length > 0 ? '\n' : '');
    return preamble;
  }

  private generateSinglePathCode(block: SinglePathBasicBlock, indent: number, queue: BasicBlock[]) {
    const preamble = this.generatePreamble(block, indent);
    if (block.next) {
      if (this.dominates(block, block.next.to)) {
        const nextBlockCode = this.generateCodeForBasicBlock(block.next.to, indent, queue);
        return `${preamble}${nextBlockCode}`;
      }

      const nextBlockLabel = this.findBasicBlockLabel(block.next.to);
      if (nextBlockLabel) {
        queue.push(block.next.to);
        if (this.visited.has(block.next.to)) {
          return `${preamble}${' '.repeat(indent)}goto(${nextBlockLabel.label});\n`;
        }
      }
    }

    return preamble;
  }

  private generateIfCode(block: IfExitBasicBlock, indent: number, queue: BasicBlock[]) {
    const conditionalCode = block.clauses
      .map((clause) => generateConditionClause(clause, this.context))
      .join(' && ');

    const [thenCode, thenQueue] = this.generateBranchCode(block, block.then);
    const [elseCode, elseQueue] = this.generateBranchCode(block, block.else);

    const lines = [
      `if (${conditionalCode}) {`,
      ...thenCode.split('\n'),
      ...(elseCode ? [`} else {`, ...elseCode.split('\n')] : []),
      '}',
    ].filter((line) => line.trim().length > 0);

    const ifStatement = lines.map((line) => `${' '.repeat(indent)}${line}`).join('\n');
    let subsequentCode = '';
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
      subsequentCode += this.generateCodeForBasicBlock(innerBlock, indent, innerQueue);
    }
    return this.generatePreamble(block, indent) + ifStatement + '\n' + subsequentCode;
  }

  private generateBranchCode(block: IfExitBasicBlock, branch?: BasicBlockEdge) {
    if (branch && this.immediatelyPostDominates(branch.to, block)) {
      return ['', [branch.to]] as const;
    }
    const branchQueue: BasicBlock[] = [];
    let branchCode = branch ? this.generateCodeForBasicBlock(branch.to, 2, branchQueue) : '';
    if (branch && branchQueue.length > 0 && !this.postDominates(branchQueue[0], branch.to)) {
      branchCode += `\n  goto(${this.findBasicBlockLabel(branchQueue[0])?.label});`;
    }
    return [branchCode, branchQueue] as const;
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

export function generateCodeForLogicResource(logic: LogicResource, wordList: WordList): string {
  const root = decompileInstructions(logic.instructions);
  const optimizedRoot = optimizeAST(root);
  const scriptGenerator = new LogicScriptGenerator(optimizedRoot, { logic, wordList });
  return scriptGenerator.generateCode() + '\n\n' + generateLogicMessages(logic);
}
