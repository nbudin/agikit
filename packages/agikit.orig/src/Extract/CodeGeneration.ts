import assertNever from 'assert-never';
import { AGICommandArgType } from '../Types/AGICommands';
import {
  LogicResource,
  LogicConditionClause,
  LogicCommand,
  LogicInstruction,
  LogicASTNode,
  LogicLabel,
} from '../Types/Logic';
import { WordList } from '../Types/WordList';
import { optimizeAST } from './ASTOptimization';
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

export function generateCodeForASTNode(
  astNode: LogicASTNode,
  context: CodeGenerationContext,
  indent = 0,
  visited?: Set<LogicASTNode>,
): string {
  const workingVisited = visited ?? new Set<LogicASTNode>();

  if (workingVisited.has(astNode)) {
    return '// WARNING: loop detected\n';
  }

  workingVisited.add(astNode);

  const indentSpaces = ' '.repeat(indent);
  const labelIfPresent = astNode.label
    ? `${' '.repeat(indent < 2 ? indent : indent - 2)}${astNode.label.label}:\n`
    : '';
  const preamble = labelIfPresent + indentSpaces;

  if (astNode.type === 'command') {
    return `${preamble}${generateLogicCommandCode(astNode, context)}\n${
      astNode.next ? generateCodeForASTNode(astNode.next, context, indent, workingVisited) : ''
    }`;
  }

  if (astNode.type === 'goto') {
    return `${preamble}goto(${astNode.jumpTarget.label?.label});`;
  }

  if (astNode.type === 'if') {
    const conditionalCode = astNode.clauses
      .map((clause) => generateConditionClause(clause, context))
      .join(' && ');

    const lines = [
      `if (${conditionalCode}) {`,
      ...(astNode.then ? [generateCodeForASTNode(astNode.then, context, 2, workingVisited)] : []),
      ...(astNode.else
        ? [`} else {`, generateCodeForASTNode(astNode.else, context, 2, workingVisited)]
        : []),
      '}',
    ];

    return `${labelIfPresent}${lines.map((line) => `${indentSpaces}${line}`).join('\n')}\n`;
  }

  return assertNever(astNode);
}

export function generateCodeForLogicResource(logic: LogicResource, wordList: WordList): string {
  const root = decompileInstructions(logic.instructions);
  const optimizedRoot = optimizeAST(root);
  return generateCodeForASTNode(optimizedRoot, { logic, wordList });
}
