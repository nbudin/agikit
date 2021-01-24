import assertNever from 'assert-never';
import { LogicConditionClause } from '../../Types/Logic';
import { isPresent } from 'ts-is-present';
import { Graph, GraphEdge, GraphNode } from '../Graphs';
import { DominatorTree } from './DominatorTree';
import {
  LogicLabel,
  LogicCommandNode,
  LogicGotoNode,
  LogicIfNode,
  LogicASTNode,
} from './LogicDecompile';

/**
 * An intermediate data structure used during decompilation.  A basic block is a consecutive set
 * of simple commands (i.e. not conditionals or gotos) followed optionally by a conditional or
 * goto. It also has references to its entry and exit vertices in the control flow graph.
 *
 * See also: https://en.wikipedia.org/wiki/Basic_block
 */
type BasicBlockCommon = {
  id: string;
  label?: LogicLabel;
  commands: LogicCommandNode[];
  entryPoints: Set<BasicBlockEdge>;
};

export type SinglePathBasicBlock = BasicBlockCommon & {
  type: 'singlePathBasicBlock';
  next?: NextBasicBlockEdge;
  metadata: {
    gotoNode?: LogicGotoNode;
  };
};

export type IfExitBasicBlock = BasicBlockCommon & {
  type: 'ifExitBasicBlock';
  clauses: LogicConditionClause[];
  then?: ThenBasicBlockEdge;
  else?: ElseBasicBlockEdge;
  metadata: {
    ifNode: LogicIfNode;
  };
};

export type BasicBlock = SinglePathBasicBlock | IfExitBasicBlock;

type BasicBlockEdgeCommon = GraphEdge<BasicBlock>;

export type NextBasicBlockEdge = Omit<BasicBlockEdgeCommon, 'from'> & {
  type: 'next';
  from: SinglePathBasicBlock;
};

export type ThenBasicBlockEdge = Omit<BasicBlockEdgeCommon, 'from'> & {
  type: 'then';
  from: IfExitBasicBlock;
};

export type ElseBasicBlockEdge = Omit<BasicBlockEdgeCommon, 'from'> & {
  type: 'else';
  from: IfExitBasicBlock;
};

export type BasicBlockEdge = NextBasicBlockEdge | ThenBasicBlockEdge | ElseBasicBlockEdge;

function getBlockExits(block: BasicBlock): BasicBlockEdge[] {
  let exits: (BasicBlockEdge | undefined)[] = [];

  if (block.type === 'singlePathBasicBlock') {
    exits = [block.next];
  } else if (block.type === 'ifExitBasicBlock') {
    exits = [block.then, block.else];
  }

  return exits.filter(isPresent);
}

export type ReverseCFGNode = GraphNode & {
  entries: ReverseCFGEdge[];
  exits: ReverseCFGEdge[];
};
export type ReverseCFGEdge = GraphEdge<ReverseCFGNode>;

export class ReverseCFG extends Graph<ReverseCFGNode> {
  getInwardEdges(node: ReverseCFGNode): ReverseCFGEdge[] {
    return node.entries;
  }

  getOutwardEdges(node: ReverseCFGNode): ReverseCFGEdge[] {
    return node.exits;
  }
}

export class BasicBlockGraph extends Graph<BasicBlock, BasicBlockEdge> {
  static fromAST(rootNode: LogicASTNode): BasicBlockGraph {
    const rootBlock = buildBasicBlocks(rootNode);
    return new BasicBlockGraph(rootBlock);
  }

  getInwardEdges(block: BasicBlock): BasicBlockEdge[] {
    return [...block.entryPoints];
  }

  getOutwardEdges(block: BasicBlock): BasicBlockEdge[] {
    return getBlockExits(block);
  }

  getNodeName(block: BasicBlock): string {
    return `${block.type.replace('BasicBlock', '')}${block.id}`;
  }

  getEdgeLabel(edge: BasicBlockEdge): string {
    return edge.type;
  }

  buildDominatorTree(): DominatorTree<BasicBlock> {
    return DominatorTree.fromCFG(this);
  }

  buildPostDominatorTree(): DominatorTree<ReverseCFGNode> {
    return DominatorTree.fromCFG(this.buildReverseCFG());
  }

  buildReverseCFG(): ReverseCFG {
    const reverseNodes = new Map<BasicBlock, ReverseCFGNode>();
    const roots = new Set<ReverseCFGNode>();
    this.depthFirstSearch((block) => {
      const reverseNode: ReverseCFGNode = {
        id: block.id,
        entries: [],
        exits: [],
      };
      reverseNodes.set(block, reverseNode);

      this.getInwardEdges(block).forEach((edge) => {
        const blockEntry = reverseNodes.get(edge.from);
        if (blockEntry && !reverseNode.exits.some((exit) => exit.to === blockEntry)) {
          const reverseEdge: ReverseCFGEdge = {
            from: reverseNode,
            to: blockEntry,
          };
          reverseNode.exits.push(reverseEdge);
          blockEntry.entries.push(reverseEdge);
        }
      });

      this.getOutwardEdges(block).forEach((edge) => {
        const blockExit = reverseNodes.get(edge.to);

        if (blockExit && !reverseNode.entries.some((entry) => entry.from === blockExit)) {
          const reverseEdge: ReverseCFGEdge = {
            from: blockExit,
            to: reverseNode,
          };
          blockExit.exits.push(reverseEdge);
          reverseNode.entries.push(reverseEdge);
        }
      });

      if (this.getOutwardEdges(block).length === 0) {
        roots.add(reverseNode);
      }
    });

    const virtualRoot: ReverseCFGNode = {
      id: 'virtualRoot',
      entries: [],
      exits: [],
    };
    roots.forEach((root) => {
      const edge: ReverseCFGEdge = {
        from: virtualRoot,
        to: root,
      };
      virtualRoot.exits.push(edge);
      root.entries.push(edge);
    });

    return new ReverseCFG(virtualRoot);
  }
}

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

function buildBasicBlocks(
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

      const singlePathBlock: SinglePathBasicBlock = {
        type: 'singlePathBasicBlock',
        id: node.id,
        label: node.label,
        commands: [node],
        entryPoints: new Set<BasicBlockEdge>(),
        metadata: {},
      };
      const nextEdge: NextBasicBlockEdge = {
        type: 'next',
        from: singlePathBlock,
        to: subsequentBlock,
      };

      singlePathBlock.next = nextEdge;

      subsequentBlock.entryPoints.add(nextEdge);
      return singlePathBlock;
    } else {
      return {
        type: 'singlePathBasicBlock',
        id: node.id,
        label: node.label,
        commands: [node],
        entryPoints: new Set<BasicBlockEdge>(),
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
      entryPoints: new Set<BasicBlockEdge>(),
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
      id: node.id,
      commands: [],
      entryPoints: new Set<BasicBlockEdge>(),
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
      id: node.id,
      clauses: node.clauses,
      commands: [],
      entryPoints: new Set<BasicBlockEdge>(),
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

export function removeEdge(edge: BasicBlockEdge): void {
  edge.to.entryPoints.delete(edge);

  switch (edge.type) {
    case 'next':
      edge.from.next = undefined;
      return;
    case 'then':
      edge.from.then = undefined;
      return;
    case 'else':
      edge.from.else = undefined;
      return;
    default:
      assertNever(edge);
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

export function replaceEdge(edge: BasicBlockEdge, newTarget: BasicBlock): void {
  if (edge.type === 'next') {
    const from = edge.from;
    removeEdge(edge);
    attachNext(from, newTarget);
  } else if (edge.type === 'then') {
    const from = edge.from;
    removeEdge(edge);
    attachThen(from, newTarget);
  } else if (edge.type === 'else') {
    const from = edge.from;
    removeEdge(edge);
    attachElse(from, newTarget);
  } else {
    assertNever(edge);
  }
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
