import { LogicCommandNode, LogicLabel } from '../Extract/Logic/LogicDecompile';
import {
  LogicCommand,
  LogicCondition,
  LogicConditionClause,
  LogicGoto,
  LogicInstruction,
} from '../Types/Logic';
import {
  BasicBlock,
  BasicBlockGraph,
  IfExitBasicBlock,
  ReverseCFGNode,
  SinglePathBasicBlock,
} from '../Extract/Logic/ControlFlowAnalysis';
import { DominatorTree } from '../Extract/Logic/DominatorTree';
import assertNever from 'assert-never';
import { add, find, flatMap, max } from 'lodash';
import { generateLabels } from '../Extract/Logic/LogicDisasm';
import { isPresent } from 'ts-is-present';

type SinglePathCompiledBlock = {
  type: 'singlePath';
  basicBlock: SinglePathBasicBlock;
  instructions: LogicInstruction[];
  next?: BasicBlock;
};

type ConditionalCompiledBlock = {
  type: 'conditional';
  basicBlock: IfExitBasicBlock;
  clauses: LogicConditionClause[];
  instructions: LogicInstruction[];
  skip: BasicBlock;
  then?: BasicBlock;
};

type CompiledBlock = SinglePathCompiledBlock | ConditionalCompiledBlock;

type StitchedBlock = LogicInstruction[];

type PostCompilationPass = (
  instructions: LogicInstruction[],
) => { instructions: LogicInstruction[]; changed: boolean };

const removeUnreachableInstructions: PostCompilationPass = (instructions) => {
  let changed = false;
  let reachable = true;
  const labels = generateLabels(instructions);
  const labelAddresses = new Set(labels.map((label) => label.address));

  const result = instructions.filter((instruction) => {
    if (labelAddresses.has(instruction.address)) {
      reachable = true;
    }

    const instructionWasReachable = reachable;

    if (instruction.type === 'goto') {
      reachable = false;
    }

    if (!instructionWasReachable) {
      changed = true;
    }

    return instructionWasReachable;
  });

  return { instructions: result, changed };
};

const removeRedundantGotoInstruction: PostCompilationPass = (instructions) => {
  const addressRemappings = new Map<number, number>();
  const findRemappedAddress = (address: number): number => {
    const remappedAddress = addressRemappings.get(address);

    if (remappedAddress == null) {
      return address;
    } else {
      return findRemappedAddress(remappedAddress);
    }
  };

  instructions.forEach((instruction, index) => {
    if (instruction.type === 'goto') {
      const nextInstruction = instructions[index + 1];
      const goToNextInstruction =
        nextInstruction && instruction.jumpAddress === nextInstruction.address;
      const goToSameAddressAsNextInstruction =
        nextInstruction &&
        nextInstruction.type === 'goto' &&
        nextInstruction.jumpAddress === instruction.jumpAddress;

      if (goToNextInstruction || goToSameAddressAsNextInstruction) {
        addressRemappings.set(instruction.address, nextInstruction.address);
      }
    }

    return true;
  });

  const result = flatMap(instructions, (instruction) => {
    if (addressRemappings.has(instruction.address)) {
      return [];
    }

    if (instruction.type === 'condition') {
      return {
        ...instruction,
        skipAddress: findRemappedAddress(instruction.skipAddress),
      };
    }

    if (instruction.type === 'goto') {
      return {
        ...instruction,
        jumpAddress: findRemappedAddress(instruction.jumpAddress),
      };
    }

    return instruction;
  });

  return { instructions: result, changed: addressRemappings.size > 0 };
};

// AGI Studio compatibility: AGI Studio can't decompile conditionals that make criss crossing jumps
// This ends up generating _slightly_ larger resources (because of the extra GOTO statements that it
// inserts) but the resulting game can be decompiled in all the AGI IDEs
const makeConditionalsSelfContained: PostCompilationPass = (instructions) => {
  let changed = false;
  const instructionIndexes = new Map(
    instructions.map((instruction, index) => [instruction.address, index]),
  );
  let maxAddress = max([...instructionIndexes.keys()]) ?? -1;
  const blockEndIndexes = [instructions.length];
  const blockEndGotos: (LogicGoto | undefined)[] = [];

  const result = flatMap(instructions, (instruction, index) => {
    let blockEnded = false;
    let outputInstruction = instruction;
    const thisBlockEndGotos: LogicGoto[] = [];
    while (index >= blockEndIndexes[0]) {
      blockEndIndexes.shift();
      blockEnded = true;
      const blockEndGoto = blockEndGotos.shift();
      if (blockEndGoto != null) {
        thisBlockEndGotos.push(blockEndGoto);
      }
    }

    if (instruction.type === 'condition') {
      const skipIndex = instructionIndexes.get(instruction.skipAddress);
      if (skipIndex == null) {
        throw new Error(
          `Conditional at ${instruction.address} skips to unknown address ${instruction.skipAddress}`,
        );
      }
      const containingBlockEnd = blockEndIndexes[0];

      if (skipIndex > containingBlockEnd) {
        const gotoAddress = maxAddress + 1;
        maxAddress = gotoAddress;
        const gotoInstruction: LogicGoto = {
          address: gotoAddress,
          jumpAddress: instruction.skipAddress,
          type: 'goto',
        };
        outputInstruction = {
          ...instruction,
          skipAddress: gotoAddress,
        };
        changed = true;
        blockEndIndexes.unshift(containingBlockEnd);
        blockEndGotos.unshift(gotoInstruction);
      } else {
        blockEndIndexes.unshift(skipIndex);
        blockEndGotos.unshift(undefined);
      }
    }

    if (blockEnded && thisBlockEndGotos.length > 0) {
      changed = true;
      return [...thisBlockEndGotos, outputInstruction];
    }

    return outputInstruction;
  });

  return { instructions: result, changed };
};

function runPostCompilationPass(
  pass: PostCompilationPass,
  instructions: LogicInstruction[],
): LogicInstruction[] {
  let workingInstructions = instructions;
  let changed = false;

  do {
    const result = pass(workingInstructions);
    changed = result.changed;
    workingInstructions = result.instructions;
  } while (changed);

  return workingInstructions;
}

function runPostCompilationPasses(
  passes: PostCompilationPass[],
  instructions: LogicInstruction[],
): LogicInstruction[] {
  return passes.reduce(
    (workingInstructions, pass) => runPostCompilationPass(pass, workingInstructions),
    instructions,
  );
}

export type LogicCompilerResult = {
  instructions: LogicInstruction[];
  labels: LogicLabel[];
};

export class LogicCompiler {
  basicBlockGraph: BasicBlockGraph;
  labels: Map<number, LogicLabel>;
  private postDominatorTree: DominatorTree<ReverseCFGNode>;
  private compiledBlocks: Map<BasicBlock, CompiledBlock>;
  private stitchedBlocks: Map<CompiledBlock, StitchedBlock>;
  private instructionsByAddress: Map<number, LogicInstruction>;

  constructor(basicBlockGraph: BasicBlockGraph, labels: LogicLabel[]) {
    this.basicBlockGraph = basicBlockGraph;
    this.compiledBlocks = new Map<BasicBlock, CompiledBlock>();
    this.labels = new Map<number, LogicLabel>(labels.map((label) => [label.address, label]));
    this.postDominatorTree = basicBlockGraph.buildPostDominatorTree();
    this.stitchedBlocks = new Map<CompiledBlock, StitchedBlock>();
    this.instructionsByAddress = new Map<number, LogicInstruction>();
  }

  compile(): LogicCompilerResult {
    this.basicBlockGraph.depthFirstSearch((block) => {
      this.compileBlock(block);
    });

    const stitchedInstructions = this.stitchBlocks(this.basicBlockGraph.root);
    const instructions = runPostCompilationPasses(
      [
        removeUnreachableInstructions,
        makeConditionalsSelfContained,
        removeRedundantGotoInstruction,
      ],
      stitchedInstructions,
    );

    return {
      instructions,
      labels: [...this.labels.values()],
    };
  }

  storeInstruction(instruction: LogicInstruction): void {
    if (this.instructionsByAddress.has(instruction.address)) {
      throw new Error(`Conflicting instruction at address ${instruction.address}`);
    }

    this.instructionsByAddress.set(instruction.address, instruction);
  }

  findFreeAddress(): number {
    return (max([...this.instructionsByAddress.keys()]) ?? 0) + 1;
  }

  compileCommandNode(node: LogicCommandNode): LogicCommand {
    const command: LogicCommand = {
      type: 'command',
      address: node.address,
      agiCommand: node.agiCommand,
      args: node.args,
    };

    this.storeInstruction(command);
    return command;
  }

  findOrBuildLabelForAddress(address: number): LogicLabel {
    if (address < 0) {
      throw new Error("Can't jump into virtual address");
    }

    const existingLabel = this.labels.get(address);
    if (existingLabel) {
      return existingLabel;
    }
    const newLabel = {
      address: address,
      label: `GeneratedLabel${address}`,
      references: [],
    };
    this.labels.set(address, newLabel);
    return newLabel;
  }

  findBlockAddress(basicBlock: BasicBlock): number {
    const block = this.compiledBlocks.get(basicBlock);
    if (!block) {
      throw new Error(`Block ${basicBlock.id} has not been compiled`);
    }

    if (block.instructions.length > 0) {
      return block.instructions[0].address;
    }

    const stitchedBlock = this.stitchedBlocks.get(block);
    if (stitchedBlock && stitchedBlock.length > 0) {
      return stitchedBlock[0].address;
    }

    if (block.type === 'singlePath') {
      if (!block.next) {
        throw new Error('Block has no instructions and no exit');
      }
      return this.findBlockAddress(block.next);
    }

    if (block.type === 'conditional') {
      throw new Error('Conditional block must begin with a conditional instruction');
    }

    assertNever(block);
  }

  stitchBlocks(basicBlock: BasicBlock): StitchedBlock {
    const block = this.compiledBlocks.get(basicBlock);
    if (!block) {
      throw new Error(`Block ${basicBlock.id} has not been compiled`);
    }

    const existing = this.stitchedBlocks.get(block);
    if (existing) {
      const blockAddress = this.findBlockAddress(basicBlock);
      this.findOrBuildLabelForAddress(blockAddress);
      const gotoInstruction: LogicGoto = {
        type: 'goto',
        address: this.findFreeAddress(),
        jumpAddress: blockAddress,
      };
      this.storeInstruction(gotoInstruction);
      return [gotoInstruction];
    }

    const stitchedBlock: StitchedBlock = [...block.instructions];
    this.stitchedBlocks.set(block, stitchedBlock);

    if (block.type === 'singlePath') {
      if (block.next) {
        stitchedBlock.push(...this.stitchBlocks(block.next));
      } else if (block.instructions.length === 0) {
        throw new Error('Block has no instructions and no exit');
      }
    } else {
      const conditionInstruction: LogicCondition = {
        type: 'condition',
        address: this.findFreeAddress(),
        clauses: block.clauses,
        skipAddress: -1,
      };
      stitchedBlock.push(conditionInstruction);
      this.storeInstruction(conditionInstruction);

      const skipInstructions = this.stitchBlocks(block.skip);
      const skipAddress = this.findBlockAddress(block.skip);
      this.findOrBuildLabelForAddress(skipAddress);
      conditionInstruction.skipAddress = skipAddress;

      stitchedBlock.push(...(block.then ? this.stitchBlocks(block.then) : []), ...skipInstructions);
    }

    return stitchedBlock;
  }

  compileBlock(block: BasicBlock): CompiledBlock {
    const existingCompiledBlock = this.compiledBlocks.get(block);
    if (existingCompiledBlock) {
      return existingCompiledBlock;
    }

    const commands: LogicInstruction[] = block.commands.map((node) =>
      this.compileCommandNode(node),
    );

    if (block.type === 'singlePathBasicBlock') {
      const compiledBlock: SinglePathCompiledBlock = {
        type: 'singlePath',
        basicBlock: block,
        instructions: commands,
        next: block.next?.to,
      };

      this.compiledBlocks.set(block, compiledBlock);
      return compiledBlock;
    }

    if (block.type === 'ifExitBasicBlock') {
      const nextBlockId = this.postDominatorTree.getImmediateDominator(block.id)?.id;
      if (!nextBlockId) {
        throw new Error("Can't find next block after if");
      }

      const nextBlock = this.basicBlockGraph.getNode(nextBlockId);
      if (!nextBlock) {
        throw new Error('Next block does not exist in tree');
      }

      const compiledBlock: ConditionalCompiledBlock = {
        type: 'conditional',
        basicBlock: block,
        clauses: block.clauses,
        instructions: commands,
        skip: block.else?.to ?? nextBlock,
        then: block.then?.to,
      };

      this.compiledBlocks.set(block, compiledBlock);
      return compiledBlock;
    }

    assertNever(block);
  }
}
