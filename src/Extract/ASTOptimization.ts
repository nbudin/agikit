import { result } from 'lodash';
import { LogicASTNode, LogicCommandNode, LogicGotoNode, LogicIfNode } from '../Types/Logic';
import { BasicBlock, replaceEdge, removeEdge, BasicBlockGraph } from './ControlFlowAnalysis';
import { NodeVisitor } from './Graphs';

export type BlockVisitor = (
  ...params: Parameters<NodeVisitor<BasicBlock>>
) => {
  changed: boolean;
};

export const removeEmptyBlock: BlockVisitor = (block) => {
  if (block.type === 'singlePathBasicBlock' && block.next && block.commands.length === 0) {
    const target = block.next.to;
    block.entryPoints.forEach((entryEdge) => {
      replaceEdge(entryEdge, target);
    });
    removeEdge(block.next);
    return { changed: true };
  }
  return { changed: false };
};

// export const restructureJumpIntoElse: BlockVisitor = (block) => {
//   if (
//     block.type === 'ifExitBasicBlock' &&
//     block.then &&
//     block.then.to.type === 'singlePathBasicBlock' &&
//     block.then.to.next &&
//     block.else &&
//     block.then.to.next.to === block.else.to
//   ) {
//     const nextBlock = block.else.to;
//     removeEdge(block.then.to.next);
//     removeEdge(block.else);
//   }
// };

export function buildASTFromBasicBlocks(
  rootBlock: BasicBlock,
  nodeIndex?: Map<BasicBlock, LogicASTNode>,
): LogicASTNode | undefined {
  const workingIndex = nodeIndex ?? new Map<BasicBlock, LogicASTNode>();

  const findOrBuildNodeForBlock = (block: BasicBlock) => {
    const existing = workingIndex.get(block);
    if (existing) {
      return existing;
    }

    const node = buildASTFromBasicBlocks(block, workingIndex);
    if (node) {
      workingIndex.set(block, node);
    }
    return node;
  };

  if (rootBlock.commands.length > 0) {
    const commandNode: LogicCommandNode = {
      ...rootBlock.commands[0],
    };
    workingIndex.set(rootBlock, commandNode);
    commandNode.next = findOrBuildNodeForBlock({
      ...rootBlock,
      commands: rootBlock.commands.slice(1),
    });
    return commandNode;
  }

  if (rootBlock.type === 'singlePathBasicBlock') {
    if (!rootBlock.next) {
      return undefined;
    }

    const gotoNode: LogicGotoNode = {
      type: 'goto',
      id: rootBlock.id,
      label: rootBlock.label,
      // fake jump target to get this in the index earlier
      jumpTarget: {
        id: 'fake',
        address: 0,
        agiCommand: { name: 'fake', argTypes: [], opcode: -1 },
        type: 'command',
        args: [],
      },
    };
    workingIndex.set(rootBlock, gotoNode);

    const jumpTarget = findOrBuildNodeForBlock(rootBlock.next.to);
    if (!jumpTarget) {
      const fromAddress = rootBlock.metadata.gotoNode?.metadata?.instructionAddress;
      const toAddress = rootBlock.metadata.gotoNode?.jumpTarget.metadata?.instructionAddress;
      throw new Error(`Invalid jump from ${fromAddress} to ${toAddress}`);
    }
    gotoNode.jumpTarget = jumpTarget;
    return gotoNode;
  }

  if (rootBlock.type === 'ifExitBasicBlock') {
    const ifNode: LogicIfNode = {
      type: 'if',
      id: rootBlock.id,
      clauses: rootBlock.clauses,
      label: rootBlock.label,
    };
    workingIndex.set(rootBlock, ifNode);

    ifNode.then = rootBlock.then ? findOrBuildNodeForBlock(rootBlock.then.to) : undefined;
    ifNode.else = rootBlock.else ? findOrBuildNodeForBlock(rootBlock.else.to) : undefined;

    return ifNode;
  }
}

export function optimizeAST(root: LogicASTNode): BasicBlockGraph {
  const basicBlockGraph = BasicBlockGraph.fromAST(root);
  [removeEmptyBlock].forEach((visitor) => {
    basicBlockGraph.depthFirstSearch((block, parent) => {
      let result: ReturnType<BlockVisitor>;
      do {
        result = visitor(block, parent);
      } while (result.changed);
    });
  });

  return basicBlockGraph;
}
