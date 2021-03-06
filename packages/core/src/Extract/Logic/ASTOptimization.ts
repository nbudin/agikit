import { LogicASTNode, LogicCommandNode, LogicGotoNode, LogicIfNode } from './LogicDecompile';
import { BasicBlock, replaceEdge, removeEdge, BasicBlockGraph } from './ControlFlowAnalysis';
import { NodeVisitor } from '../Graphs';

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

export const concatenateLinearBlocks: BlockVisitor = (block) => {
  if (block.type === 'singlePathBasicBlock' && block.entryPoints.size === 1) {
    const inwardEdge = [...block.entryPoints.values()][0];
    const previousBlock = inwardEdge.from;
    if (previousBlock.type === 'singlePathBasicBlock') {
      previousBlock.commands.push(...block.commands);
      if (block.next) {
        replaceEdge(inwardEdge, block.next.to);
        removeEdge(block.next);
      } else {
        removeEdge(inwardEdge);
      }
      return { changed: true };
    }
  }
  return { changed: false };
};

// export const removeJumpIntoReturn: BlockVisitor = (block) => {
//   if (block.type === 'singlePathBasicBlock' && block.next) {
//     const target = block.next.to;
//     const nextCommand = target.commands[0];
//     if (nextCommand?.agiCommand.name === 'return') {
//       block.commands.push({
//         ...nextCommand,
//         id: `removedJumpIntoReturnFrom${block.id}`,
//       });
//       removeEdge(block.next);
//       return { changed: true };
//     }
//   }
//   return { changed: false };
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
  [removeEmptyBlock, concatenateLinearBlocks].forEach((visitor) => {
    let changed: boolean;
    do {
      changed = false;
      basicBlockGraph.depthFirstSearch((block, parent) => {
        const result = visitor(block, parent);
        if (result.changed) {
          changed = true;
        }
      });
    } while (changed);
  });

  return basicBlockGraph;
}
