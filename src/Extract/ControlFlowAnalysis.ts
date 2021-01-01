import assertNever from 'assert-never';
import {
  LogicLabel,
  LogicCommandNode,
  LogicGotoNode,
  LogicConditionClause,
  LogicIfNode,
  LogicASTNode,
} from '../Types/Logic';
import { isPresent } from 'ts-is-present';
import { remove } from 'lodash';

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
  entryPoints: Set<BasicBlockVertex>;
};

export type SinglePathBasicBlock = BasicBlockCommon & {
  type: 'singlePathBasicBlock';
  next?: NextBasicBlockVertex;
  metadata: {
    gotoNode?: LogicGotoNode;
  };
};

export type IfExitBasicBlock = BasicBlockCommon & {
  type: 'ifExitBasicBlock';
  clauses: LogicConditionClause[];
  then?: ThenBasicBlockVertex;
  else?: ElseBasicBlockVertex;
  metadata: {
    ifNode: LogicIfNode;
  };
};

export type BasicBlock = SinglePathBasicBlock | IfExitBasicBlock;

type BasicBlockVertexCommon = {
  from: BasicBlock;
  to: BasicBlock;
};

export type NextBasicBlockVertex = Omit<BasicBlockVertexCommon, 'from'> & {
  type: 'next';
  from: SinglePathBasicBlock;
};

export type ThenBasicBlockVertex = Omit<BasicBlockVertexCommon, 'from'> & {
  type: 'then';
  from: IfExitBasicBlock;
};

export type ElseBasicBlockVertex = Omit<BasicBlockVertexCommon, 'from'> & {
  type: 'else';
  from: IfExitBasicBlock;
};

export type BasicBlockVertex = NextBasicBlockVertex | ThenBasicBlockVertex | ElseBasicBlockVertex;

export function formatKeyValuePairs<T>(obj: T | undefined, keys: (keyof T)[]): string {
  if (obj == null) {
    return '';
  }

  const pairs = keys
    .map((key) => (obj[key] == null ? undefined : `${key}=${obj[key]}`))
    .filter(isPresent);

  if (pairs.length > 0) {
    return `[${pairs.join(', ')}]`;
  }

  return '';
}

export function basicBlockDebugName(block: BasicBlock): string {
  if (block.type === 'singlePathBasicBlock') {
    if (block.commands.length > 0) {
      return `singlePath${block.commands[0].address}`;
    } else if (block.metadata.gotoNode?.metadata?.instructionAddress) {
      return `jumpFrom${block.metadata.gotoNode.metadata.instructionAddress}`;
    } else if (block.next) {
      return `jumpFromUnknown`;
    }

    return 'empty';
  }

  if (block.type === 'ifExitBasicBlock') {
    return `conditional${block.metadata.ifNode.metadata?.instructionAddress ?? 'Unknown'}`;
  }

  assertNever(block);
}

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
        const singlePathBlock: SinglePathBasicBlock = {
          type: 'singlePathBasicBlock',
          label: node.label,
          commands: [node],
          entryPoints: new Set<BasicBlockVertex>(),
          metadata: {},
        };
        const nextVertex: NextBasicBlockVertex = {
          type: 'next',
          from: singlePathBlock,
          to: subsequentBlock,
        };
        singlePathBlock.next = nextVertex;

        subsequentBlock.entryPoints.add(nextVertex);
        return singlePathBlock;
      }

      return {
        ...subsequentBlock,
        label: node.label,
        commands: [node, ...subsequentBlock.commands],
      };
    } else {
      return {
        type: 'singlePathBasicBlock',
        label: node.label,
        commands: [node],
        entryPoints: new Set<BasicBlockVertex>(),
        metadata: {},
      };
    }
  }

  if (node.type === 'goto') {
    // We need to add this block to the index before fully building it out, in order to
    // be able to detect cycles (because you can infinite-loop with goto instructions).
    // In order to satisfy the type checker, we'll create a fake self-referential next
    // vertex and then replace it once we know where we're actually jumping to.
    const fakeJumpTargetPartial: Partial<SinglePathBasicBlock> = {
      type: 'singlePathBasicBlock',
      commands: [],
      entryPoints: new Set<BasicBlockVertex>(),
      metadata: {},
    };
    fakeJumpTargetPartial.next = {
      type: 'next',
      from: fakeJumpTargetPartial as SinglePathBasicBlock,
      to: fakeJumpTargetPartial as SinglePathBasicBlock,
    };
    const fakeJumpTarget = fakeJumpTargetPartial as SinglePathBasicBlock;

    const gotoBlock: SinglePathBasicBlock = {
      type: 'singlePathBasicBlock',
      commands: [],
      entryPoints: new Set<BasicBlockVertex>(),
      next: fakeJumpTarget.next,
      metadata: {
        gotoNode: node,
      },
    };

    workingIndex.set(node, gotoBlock);

    if (node.jumpTarget === node) {
      gotoBlock.next = {
        type: 'next',
        from: gotoBlock,
        to: gotoBlock,
      };
    } else {
      gotoBlock.next = {
        type: 'next',
        from: gotoBlock,
        to: findOrBuildBlocksForNode(node.jumpTarget),
      };
    }

    gotoBlock.next.to.entryPoints.add(gotoBlock.next);
    return gotoBlock;
  }

  if (node.type === 'if') {
    const ifExitBlock: IfExitBasicBlock = {
      type: 'ifExitBasicBlock',
      clauses: node.clauses,
      commands: [],
      entryPoints: new Set<BasicBlockVertex>(),
      label: node.label,
      metadata: {
        ifNode: node,
      },
    };

    workingIndex.set(node, ifExitBlock);

    if (node.then) {
      ifExitBlock.then = {
        type: 'then',
        from: ifExitBlock,
        to: findOrBuildBlocksForNode(node.then),
      };
      ifExitBlock.then.to.entryPoints.add(ifExitBlock.then);
    }

    if (node.else) {
      ifExitBlock.else = {
        type: 'else',
        from: ifExitBlock,
        to: findOrBuildBlocksForNode(node.else),
      };
      ifExitBlock.else.to.entryPoints.add(ifExitBlock.else);
    }

    return ifExitBlock;
  }

  assertNever(node);
}

export function removeVertex(vertex: BasicBlockVertex): void {
  vertex.to.entryPoints.delete(vertex);

  switch (vertex.type) {
    case 'next':
      vertex.from.next = undefined;
      return;
    case 'then':
      vertex.from.then = undefined;
      return;
    case 'else':
      vertex.from.else = undefined;
      return;
    default:
      assertNever(vertex);
  }
}

export function attachNext(block: SinglePathBasicBlock, nextBlock: BasicBlock): void {
  if (block.next) {
    throw new Error(
      `Can't attach ${basicBlockDebugName(nextBlock)} to ${basicBlockDebugName(
        block,
      )} because it already has ${basicBlockDebugName(block.next.to)} attached`,
    );
  }

  block.next = {
    type: 'next',
    from: block,
    to: nextBlock,
  };
  block.next.to.entryPoints.add(block.next);
}

export function attachThen(block: IfExitBasicBlock, nextBlock: BasicBlock): void {
  if (block.then) {
    throw new Error(
      `Can't attach ${basicBlockDebugName(nextBlock)} to then branch of ${basicBlockDebugName(
        block,
      )} because it already has ${basicBlockDebugName(block.then.to)} attached`,
    );
  }

  block.then = {
    type: 'then',
    from: block,
    to: nextBlock,
  };
  block.then.to.entryPoints.add(block.then);
}

export function attachElse(block: IfExitBasicBlock, nextBlock: BasicBlock): void {
  if (block.else) {
    throw new Error(
      `Can't attach ${basicBlockDebugName(nextBlock)} to else branch of ${basicBlockDebugName(
        block,
      )} because it already has ${basicBlockDebugName(block.else.to)} attached`,
    );
  }

  block.else = {
    type: 'else',
    from: block,
    to: nextBlock,
  };
  block.else.to.entryPoints.add(block.else);
}

export function replaceVertex(vertex: BasicBlockVertex, newTarget: BasicBlock): void {
  if (vertex.type === 'next') {
    const from = vertex.from;
    removeVertex(vertex);
    attachNext(from, newTarget);
  } else if (vertex.type === 'then') {
    const from = vertex.from;
    removeVertex(vertex);
    attachThen(from, newTarget);
  } else if (vertex.type === 'else') {
    const from = vertex.from;
    removeVertex(vertex);
    attachElse(from, newTarget);
  } else {
    assertNever(vertex);
  }
}

export function getBlockExits(block: BasicBlock): BasicBlockVertex[] {
  let exits: (BasicBlockVertex | undefined)[] = [];

  if (block.type === 'singlePathBasicBlock') {
    exits = [block.next];
  } else if (block.type === 'ifExitBasicBlock') {
    exits = [block.then, block.else];
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
