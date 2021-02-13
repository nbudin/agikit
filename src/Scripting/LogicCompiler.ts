import { LogicCommandNode, LogicLabel } from '../Extract/Logic/LogicDecompile';
import { LogicCommand, LogicInstruction } from '../Types/Logic';
import { BasicBlock, BasicBlockGraph, ReverseCFGNode } from '../Extract/Logic/ControlFlowAnalysis';
import { DominatorTree } from '../Extract/Logic/DominatorTree';
import assertNever from 'assert-never';

type CompiledBlock = {
  instructions: LogicInstruction[];
  address: number;
};

export type LogicCompilerResult = {
  instructions: LogicInstruction[];
  labels: LogicLabel[];
};

export class LogicCompiler {
  basicBlockGraph: BasicBlockGraph;
  labels: Map<number, LogicLabel>;
  private postDominatorTree: DominatorTree<ReverseCFGNode>;
  private compiledBlocks: Map<BasicBlock, CompiledBlock>;

  constructor(basicBlockGraph: BasicBlockGraph, labels: LogicLabel[]) {
    this.basicBlockGraph = basicBlockGraph;
    this.compiledBlocks = new Map<BasicBlock, CompiledBlock>();
    this.labels = new Map<number, LogicLabel>(labels.map((label) => [label.address, label]));
    this.postDominatorTree = basicBlockGraph.buildPostDominatorTree();
  }

  compile(): LogicCompilerResult {
    const { instructions } = this.compileBlock(this.basicBlockGraph.root, 0);
    return {
      instructions,
      labels: [...this.labels.values()],
    };
  }

  compileCommandNode(node: LogicCommandNode): LogicCommand {
    return {
      type: 'command',
      address: node.address,
      agiCommand: node.agiCommand,
      args: node.args,
    };
  }

  findOrBuildLabelForAddress(address: number): LogicLabel {
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

  traverseTo(block: BasicBlock | undefined, edgeAddress: number): CompiledBlock {
    if (!block) {
      return { instructions: [], address: -1 };
    }

    const existingCompiledBlock = this.compiledBlocks.get(block);

    if (existingCompiledBlock) {
      if (block.entryPoints.size === 1) {
        return existingCompiledBlock;
      }

      this.findOrBuildLabelForAddress(existingCompiledBlock.address);
      return {
        instructions: [
          {
            type: 'goto',
            address: edgeAddress,
            jumpAddress: existingCompiledBlock.address,
          },
        ],
        address: edgeAddress,
      };
    } else {
      if (block.entryPoints.size === 1) {
        return this.compileBlock(block, edgeAddress);
      }

      const targetBlock = this.compileBlock(block, edgeAddress);
      this.findOrBuildLabelForAddress(targetBlock.address);
      return targetBlock;
    }
  }

  compileBlock(block: BasicBlock, address: number): CompiledBlock {
    const existingCompiledBlock = this.compiledBlocks.get(block);
    if (existingCompiledBlock) {
      return existingCompiledBlock;
    }

    const commands: LogicInstruction[] = block.commands.map((node) =>
      this.compileCommandNode(node),
    );
    const lastCommandAddress =
      commands.length > 0 ? commands[commands.length - 1].address : address;

    this.compiledBlocks.set(block, { instructions: commands, address });

    if (block.type === 'singlePathBasicBlock') {
      return {
        instructions: [
          ...commands,
          ...this.traverseTo(block.next?.to, lastCommandAddress + 1).instructions,
        ],
        address,
      };
    }

    if (block.type === 'ifExitBasicBlock') {
      const nextBlockId = this.postDominatorTree.getImmediateDominator(block.id)?.id;
      if (!nextBlockId) {
        throw new Error("Can't find next block after if");
      }

      const nextBlock = this.basicBlockGraph.getNode(nextBlockId);

      const nextBlockIsNew = nextBlock && !this.compiledBlocks.has(nextBlock);
      const nextBlockAddress =
        nextBlock && (nextBlock?.commands ?? []).length > 0
          ? nextBlock.commands[0].address - 1
          : lastCommandAddress + 3;

      const compiledNextBlock = this.traverseTo(nextBlock, nextBlockAddress);

      const compiledElseBlock = block.else
        ? this.traverseTo(block.else.to, lastCommandAddress + 2)
        : undefined;
      const compiledThenBlock = block.then
        ? this.traverseTo(block.then.to, lastCommandAddress + 2)
        : undefined;
      const skipAddress = compiledElseBlock ? compiledElseBlock.address : compiledNextBlock.address;
      this.findOrBuildLabelForAddress(skipAddress);
      commands.push({
        type: 'condition',
        address: lastCommandAddress + 1,
        clauses: block.clauses,
        skipAddress,
      });
      // I think this is unnecessary but I'm not positive
      this.compiledBlocks.set(block, { instructions: commands, address });

      return {
        instructions: [
          ...commands,
          ...(compiledThenBlock?.instructions ?? []),
          ...(compiledElseBlock?.instructions ?? []),
          // If we've already seen the next block before getting here, it will result in a redundant
          // goto statement. In that case we just leave it out.
          ...(nextBlockIsNew ? compiledNextBlock.instructions : []),
        ],
        address,
      };
    }

    assertNever(block);
  }
}
