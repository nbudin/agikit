import { LogicCommandNode, LogicLabel } from '../Extract/Logic/LogicDecompile';
import { LogicCommand, LogicInstruction } from '../Types/Logic';
import { BasicBlock, BasicBlockGraph, ReverseCFGNode } from '../Extract/Logic/ControlFlowAnalysis';
import { DominatorTree } from '../Extract/Logic/DominatorTree';
import assertNever from 'assert-never';

type CompiledBlock = LogicInstruction[];

export type LogicCompilerResult = {
  instructions: LogicInstruction[];
  labels: LogicLabel[];
};

export class LogicCompiler {
  basicBlockGraph: BasicBlockGraph;
  labels: Map<number, LogicLabel>;
  private postDominatorTree: DominatorTree<ReverseCFGNode>;
  private compiledBlocks: Map<BasicBlock, CompiledBlock>;

  constructor(basicBlockGraph: BasicBlockGraph) {
    this.basicBlockGraph = basicBlockGraph;
    this.compiledBlocks = new Map<BasicBlock, CompiledBlock>();
    this.labels = new Map<number, LogicLabel>();
    this.postDominatorTree = basicBlockGraph.buildPostDominatorTree();
  }

  compile(): LogicCompilerResult {
    const instructions = this.compileBlock(this.basicBlockGraph.root, 0);
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

  traverseTo(block: BasicBlock | undefined, edgeAddress: number): CompiledBlock {
    if (!block) {
      return [];
    }
    if (block.entryPoints.size === 1) {
      return this.compileBlock(block, edgeAddress);
    }

    const existingCompiledBlock = this.compiledBlocks.get(block);

    if (existingCompiledBlock) {
      return [
        {
          type: 'goto',
          address: edgeAddress,
          jumpAddress: existingCompiledBlock[0].address,
        },
      ];
    } else {
      const targetBlock = this.compileBlock(block, edgeAddress);
      const targetBlockLabel: LogicLabel = {
        address: targetBlock[0].address,
        label: `Label${targetBlock[0].address}`,
        references: [],
      };
      this.labels.set(targetBlockLabel.address, targetBlockLabel);
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

    if (block.type === 'singlePathBasicBlock') {
      this.compiledBlocks.set(block, commands);
      return [...commands, ...this.traverseTo(block.next?.to, lastCommandAddress + 1)];
    }

    if (block.type === 'ifExitBasicBlock') {
      const nextBlockId = this.postDominatorTree.getImmediateDominator(block.id)?.id;
      if (!nextBlockId) {
        throw new Error("Can't find next block after if");
      }

      const nextBlock = this.basicBlockGraph.getNode(nextBlockId);
      const compiledNextBlock = this.traverseTo(
        nextBlock,
        (nextBlock?.commands ?? [])[0].address - 1,
      );
      const compiledElseBlock = block.else
        ? this.traverseTo(block.else.to, lastCommandAddress + 2)
        : undefined;
      const compiledThenBlock = block.then
        ? this.traverseTo(block.then.to, lastCommandAddress + 2)
        : [];
      commands.push({
        type: 'condition',
        address: lastCommandAddress + 1,
        clauses: block.clauses,
        skipAddress: compiledElseBlock
          ? compiledElseBlock[0].address
          : compiledNextBlock[0].address,
      });
      this.compiledBlocks.set(block, commands);

      return [
        ...commands,
        ...compiledThenBlock,
        ...(compiledElseBlock ?? []),
        ...compiledNextBlock,
      ];
    }

    assertNever(block);
  }
}
