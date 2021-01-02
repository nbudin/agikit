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
import { BasicBlock, BasicBlockGraph } from './ControlFlowAnalysis';
import { DominatorTree } from './DominatorTree';
import { decompileInstructions } from './LogicDecompile';
import { generateLabels } from './LogicDisasm';

export type CodeGenerationContext = {
  logic: LogicResource;
  wordList: WordList;
};

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
      return [...(context.wordList.get(value)?.values() ?? [])][0] ?? `w${value}`;
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

export function generateLogicAsm(logic: LogicResource, wordList: WordList): string {
  const labels = generateLabels(logic.instructions);
  const asmCode = logic.instructions.map((instruction) =>
    generateLogicAsmInstructionWithPossibleLabel(instruction, labels, { logic, wordList }),
  );
  const messages = logic.messages.map((message, index) => `#message ${index + 1} "${message}"`);

  return `${asmCode.join('\n')}\n\n// messages\n${messages.join('\n')}\n`;
}

function findBasicBlockLabel(block: BasicBlock): LogicLabel | undefined {
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

function generateCodeForBasicBlock(
  block: BasicBlock,
  context: CodeGenerationContext,
  dominatorTree: DominatorTree<BasicBlock>,
  indent: number,
  visited: Set<BasicBlock>,
  queue: BasicBlock[],
): string {
  const workingVisited = visited ?? new Set<BasicBlock>();

  if (workingVisited.has(block)) {
    return '// WARNING: loop detected\n';
  }

  workingVisited.add(block);

  const indentSpaces = ' '.repeat(indent);
  const blockLabel = findBasicBlockLabel(block);
  const labelIfPresent = blockLabel
    ? `${' '.repeat(indent < 2 ? indent : indent - 2)}${blockLabel.label}:\n`
    : '';
  const preamble = labelIfPresent + indentSpaces;

  const commandSection = block.commands
    .map((command) => `${preamble}${generateLogicCommandCode(command, context)}`)
    .join('\n');

  if (block.type === 'singlePathBasicBlock') {
    if (block.next) {
      const nextBlockLabel = findBasicBlockLabel(block.next.to);
      if (nextBlockLabel) {
        queue.push(block.next.to);
        return `${commandSection}\n${preamble}goto(${nextBlockLabel.label});`;
      }
    }

    return commandSection;
  }

  if (block.type === 'ifExitBasicBlock') {
    const conditionalCode = block.clauses
      .map((clause) => generateConditionClause(clause, context))
      .join(' && ');

    const lines = [
      `if (${conditionalCode}) {`,
      ...(block.then
        ? generateCodeForBasicBlock(
            block.then.to,
            context,
            dominatorTree,
            2,
            workingVisited,
            queue,
          ).split('\n')
        : []),
      ...(block.else
        ? [
            `} else {`,
            ...generateCodeForBasicBlock(
              block.else.to,
              context,
              dominatorTree,
              2,
              workingVisited,
              queue,
            ).split('\n'),
          ]
        : []),
      '}',
    ];

    const ifStatement = labelIfPresent + lines.map((line) => `${indentSpaces}${line}`).join('\n');
    return ifStatement;
  }

  return assertNever(block);
}

export function generateCodeForBasicBlockGraph(
  graph: BasicBlockGraph,
  dominatorTree: DominatorTree<BasicBlock>,
  context: CodeGenerationContext,
): string {
  const queue: BasicBlock[] = [graph.root];
  const visited = new Set<BasicBlock>();
  let code = '';

  while (queue.length > 0) {
    const block = queue.shift();
    if (!block || visited.has(block)) {
      continue;
    }
    code += generateCodeForBasicBlock(block, context, dominatorTree, 0, visited, queue);
  }

  return code;
}

export function generateCodeForLogicResource(logic: LogicResource, wordList: WordList): string {
  const root = decompileInstructions(logic.instructions);
  const optimizedRoot = optimizeAST(root);
  const dominatorTree = DominatorTree.fromCFG(optimizedRoot);
  return generateCodeForBasicBlockGraph(optimizedRoot, dominatorTree, { logic, wordList });
}
