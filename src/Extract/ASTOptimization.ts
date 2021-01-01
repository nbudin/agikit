import { notEqual } from 'assert';
import assertNever from 'assert-never';
import { isPresent } from 'ts-is-present';
import {
  LogicASTNode,
  LogicCommandNode,
  LogicConditionClause,
  LogicGotoNode,
  LogicIfNode,
  LogicLabel,
} from '../Types/Logic';

/**
 * An intermediate data structure used during decompilation.  A basic block is a consecutive set
 * of simple commands (i.e. not conditionals or gotos) followed optionally by a conditional or
 * goto. It also has references to its entry and exit vertices in the control flow graph.
 *
 * See also: https://en.wikipedia.org/wiki/Basic_block
 */
type BasicBlockCommon = {
  label?: LogicLabel;
  commands: LogicCommandNode[];
  entryPoints: Set<BasicBlock>;
};

export type SimpleBasicBlock = BasicBlockCommon & {
  type: 'simpleBasicBlock';
  next?: BasicBlock;
};

export type GotoExitBasicBlock = BasicBlockCommon & {
  type: 'gotoExitBasicBlock';
  jumpTarget: BasicBlock;
};

export type IfExitBasicBlock = BasicBlockCommon & {
  type: 'ifExitBasicBlock';
  clauses: LogicConditionClause[];
  then?: BasicBlock;
  else?: BasicBlock;
  next?: BasicBlock;
};

export type BasicBlock = SimpleBasicBlock | GotoExitBasicBlock | IfExitBasicBlock;

export function buildBasicBlocks(
  node: LogicASTNode,
  blockIndex?: Map<LogicASTNode, BasicBlock>,
): BasicBlock {
  const workingIndex = blockIndex ?? new Map<LogicASTNode, BasicBlock>();

  const findOrBuildBlocksForNode = (node: LogicASTNode) => {
    const existing = workingIndex.get(node);
    if (existing) {
      return existing;
    }

    const block = buildBasicBlocks(node, workingIndex);
    workingIndex.set(node, block);
    return block;
  };

  if (node.type === 'command') {
    if (node.next) {
      const subsequentBlock = findOrBuildBlocksForNode(node.next);
      if (subsequentBlock.label) {
        const simpleBlock: SimpleBasicBlock = {
          type: 'simpleBasicBlock',
          label: node.label,
          commands: [node],
          entryPoints: new Set<BasicBlock>(),
          next: subsequentBlock,
        };

        subsequentBlock.entryPoints.add(simpleBlock);
        return simpleBlock;
      }

      return {
        ...subsequentBlock,
        label: node.label,
        commands: [node, ...subsequentBlock.commands],
      };
    } else {
      return {
        type: 'simpleBasicBlock',
        label: node.label,
        commands: [node],
        entryPoints: new Set<BasicBlock>(),
      };
    }
  }

  if (node.type === 'goto') {
    const gotoExitBlock: GotoExitBasicBlock = {
      type: 'gotoExitBasicBlock',
      commands: [],
      entryPoints: new Set<BasicBlock>(),
      // just set it to _something_ so we can lazy-resolve this
      jumpTarget: { type: 'simpleBasicBlock', commands: [], entryPoints: new Set<BasicBlock>() },
    };

    workingIndex.set(node, gotoExitBlock);

    if (node.jumpTarget === node) {
      gotoExitBlock.jumpTarget = gotoExitBlock;
    } else {
      gotoExitBlock.jumpTarget = findOrBuildBlocksForNode(node.jumpTarget);
    }

    gotoExitBlock.jumpTarget.entryPoints.add(gotoExitBlock);
    return gotoExitBlock;
  }

  if (node.type === 'if') {
    const ifExitBlock: IfExitBasicBlock = {
      type: 'ifExitBasicBlock',
      clauses: node.clauses,
      commands: [],
      entryPoints: new Set<BasicBlock>(),
      label: node.label,
      then: node.then ? findOrBuildBlocksForNode(node.then) : undefined,
      else: node.else ? findOrBuildBlocksForNode(node.else) : undefined,
      next: node.next ? findOrBuildBlocksForNode(node.next) : undefined,
    };

    [ifExitBlock.then, ifExitBlock.else, ifExitBlock.next].forEach((subsequentBlock) => {
      if (subsequentBlock) {
        subsequentBlock.entryPoints.add(ifExitBlock);
      }
    });

    return ifExitBlock;
  }

  assertNever(node);
}

export function getBlockExits(block: BasicBlock): BasicBlock[] {
  let exits: (BasicBlock | undefined)[] = [];

  if (block.type === 'simpleBasicBlock') {
    exits = [block.next];
  } else if (block.type === 'gotoExitBasicBlock') {
    exits = [block.jumpTarget];
  } else if (block.type === 'ifExitBasicBlock') {
    exits = [block.then, block.else, block.next];
  }

  return exits.filter(isPresent);
}

// function isBlockReachableFrom(start: BasicBlock, finish: BasicBlock, visited?: Set<BasicBlock>) {
//   if (start === finish) {
//     return true;
//   }

//   const visitedSet = visited ?? new Set<BasicBlock>();
//   visitedSet.add(start);
//   return getBlockExits(start).some((exitBlock) =>
//     isBlockReachableFrom(exitBlock, finish, visitedSet),
//   );
// }

export type BlockVisitor = (block: BasicBlock) => void;

export const reorganizeUnlessGoto: BlockVisitor = (block) => {
  if (
    block.type === 'ifExitBasicBlock' &&
    !block.then &&
    block.else?.type === 'gotoExitBasicBlock' &&
    block.else.commands.length === 0
  ) {
    const newNext = block.else.jumpTarget;
    block.then = block.next;
    block.next = newNext;
    block.else = undefined;

    [...newNext.entryPoints].forEach((entryPoint) => {
      if (entryPoint.type === 'simpleBasicBlock') {
        entryPoint.next = undefined;
        newNext.entryPoints.delete(entryPoint);
      } else if (entryPoint.type === 'ifExitBasicBlock' && entryPoint.next === newNext) {
        entryPoint.next = undefined;
        newNext.entryPoints.delete(entryPoint);
      }
    });
  }
};

export function dfsBasicBlocks(
  block: BasicBlock,
  visitor: BlockVisitor,
  visited: Set<BasicBlock>,
): void {
  if (visited.has(block)) {
    return;
  }

  visitor(block);
  visited.add(block);
  getBlockExits(block).forEach((exitNode) => dfsBasicBlocks(exitNode, visitor, visited));
}

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

  if (rootBlock.type === 'simpleBasicBlock') {
    if (rootBlock.next) {
      return findOrBuildNodeForBlock(rootBlock.next);
    } else {
      return undefined;
    }
  }

  if (rootBlock.type === 'gotoExitBasicBlock') {
    const gotoNode: LogicGotoNode = {
      type: 'goto',
      label: rootBlock.label,
      // fake jump target to get this in the index earlier
      jumpTarget: {
        address: 0,
        agiCommand: { name: 'fake', argTypes: [], opcode: -1 },
        type: 'command',
        args: [],
      },
    };
    workingIndex.set(rootBlock, gotoNode);

    const jumpTarget = findOrBuildNodeForBlock(rootBlock.jumpTarget);
    if (!jumpTarget) {
      throw new Error('Invalid jump');
    }
    gotoNode.jumpTarget = jumpTarget;
    return gotoNode;
  }

  if (rootBlock.type === 'ifExitBasicBlock') {
    const ifNode: LogicIfNode = {
      type: 'if',
      clauses: rootBlock.clauses,
      label: rootBlock.label,
    };
    workingIndex.set(rootBlock, ifNode);

    ifNode.then = rootBlock.then ? findOrBuildNodeForBlock(rootBlock.then) : undefined;
    ifNode.else = rootBlock.else ? findOrBuildNodeForBlock(rootBlock.else) : undefined;
    ifNode.next = rootBlock.next ? findOrBuildNodeForBlock(rootBlock.next) : undefined;

    return ifNode;
  }
}

export function optimizeAST(root: LogicASTNode): LogicASTNode {
  const rootBlock = buildBasicBlocks(root);
  [reorganizeUnlessGoto].forEach((visitor) =>
    dfsBasicBlocks(rootBlock, visitor, new Set<BasicBlock>()),
  );

  return buildASTFromBasicBlocks(rootBlock)!;
}
