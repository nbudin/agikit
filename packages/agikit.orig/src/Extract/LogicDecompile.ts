import assertNever from 'assert-never';
import { flatMap, max } from 'lodash';
import {
  LogicConditionClause,
  LogicCommand,
  LogicGoto,
  LogicInstruction,
  LogicCondition,
} from '../Types/Logic';
import { generateLabels, LogicLabel } from './LogicDisasm';

export type LogicCommandNode = LogicCommand & {
  label?: LogicLabel;
  next?: LogicASTNode;
};

export type LogicIfNode = {
  type: 'if';
  clauses: LogicConditionClause[];
  then: LogicASTNode[];
  else: LogicASTNode[];
  label?: LogicLabel;
  next?: LogicASTNode;
};

export type LogicGotoNode = {
  type: 'goto';
  jumpTarget: LogicASTNode;
  label?: LogicLabel;
};

export type LogicASTNode = LogicIfNode | LogicGotoNode | LogicCommandNode;

/**
 * An intermediate data structure used during decompilation.  A section is a consecutive block
 * of simple commands (i.e. not conditionals or gotos) followed optionally by a conditional or
 * goto. It also has references to the previous and next blocks in the logic so the decompiler
 * can figure out where it came from and where it might exit to.
 *
 * See also: https://en.wikipedia.org/wiki/Basic_block
 */
export type BasicBlock = {
  label?: LogicLabel;
  startAddress: number;
  instructions: LogicCommand[];
  endingJump?: LogicCondition | LogicGoto;
  previousBlock?: BasicBlock;
  nextBlock?: BasicBlock;
  entryPoints: Set<BasicBlock>;
  conditionalSkipTarget?: BasicBlock;
  gotoJumpTarget?: BasicBlock;
};

function splitBasicBlocks(instructions: LogicInstruction[], labels: LogicLabel[]): BasicBlock[] {
  const blocks: BasicBlock[] = [];
  let currentBlock: BasicBlock | undefined;
  let previousBlock: BasicBlock | undefined;

  instructions.forEach((instruction) => {
    const label = labels.find((label) => label.address === instruction.address);

    if (label && currentBlock) {
      // split blocks at labeled instructions
      blocks.push(currentBlock);
      previousBlock = currentBlock;
      currentBlock = undefined;
    }

    if (!currentBlock) {
      currentBlock = {
        startAddress: instruction.address,
        instructions: [],
        entryPoints: new Set<BasicBlock>(),
        label,
        previousBlock,
      };
      if (previousBlock && !previousBlock.gotoJumpTarget) {
        previousBlock.nextBlock = currentBlock;
      }
    }

    if (instruction.type === 'command') {
      currentBlock.instructions.push(instruction);
    } else {
      currentBlock.endingJump = instruction;
      blocks.push(currentBlock);
      previousBlock = currentBlock;
      currentBlock = undefined;
    }
  });

  if (currentBlock) {
    blocks.push(currentBlock);
  }

  return blocks;
}

function analyzeBlockDependencies(blocks: BasicBlock[]): void {
  const blockIndex = new Map<number, BasicBlock>(
    blocks.map((block) => [block.startAddress, block]),
  );

  blocks.forEach((block) => {
    if (block.previousBlock) {
      block.entryPoints.add(block.previousBlock);
    }

    if (block.endingJump) {
      if (block.endingJump.type === 'condition') {
        const skipTarget = blockIndex.get(block.endingJump.skipAddress);
        if (!skipTarget) {
          throw new Error(
            `Invalid skip address: ${block.endingJump.skipAddress} at ${block.endingJump.address}`,
          );
        }

        skipTarget.entryPoints.add(block);
        block.conditionalSkipTarget = skipTarget;
      } else if (block.endingJump.type === 'goto') {
        const jumpTarget = blockIndex.get(block.endingJump.jumpAddress);
        if (!jumpTarget) {
          throw new Error(`Invalid jump address: ${block.endingJump.jumpAddress}`);
        }

        jumpTarget.entryPoints.add(block);
        block.gotoJumpTarget = jumpTarget;
      }
    }

    if (block.nextBlock && (!block.endingJump || block.endingJump.type !== 'goto')) {
      block.nextBlock.entryPoints.add(block);
    }
  });
}

function buildBasicBlocks(instructions: LogicInstruction[]) {
  const labels = generateLabels(instructions);
  const blocks = splitBasicBlocks(instructions, labels);
  analyzeBlockDependencies(blocks);
  return blocks;
}

// function getBlockExits(block: BasicBlock): BasicBlock[] {
//   return [block.nextBlock, block.gotoJumpTarget, block.conditionalSkipTarget].filter(isPresent);
// }

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

type UnresolvedIfNode = {
  type: 'unresolvedIf';
  address: number;
  clauses: LogicConditionClause[];
  elseGoto: BasicBlock;
  nextBlock?: BasicBlock;
};

type UnresolvedGotoNode = {
  type: 'unresolvedGoto';
  address: number;
  jumpTargetAddress: number;
};

type UnresolvedLogicASTNode = LogicCommand | UnresolvedIfNode | UnresolvedGotoNode;

function decompileBlock(block: BasicBlock): UnresolvedLogicASTNode[] {
  const unresolvedNodes: UnresolvedLogicASTNode[] = [...block.instructions];

  if (block.endingJump?.type === 'condition') {
    const condition = block.endingJump;

    const ifNode: UnresolvedIfNode = {
      type: 'unresolvedIf',
      address: condition.address,
      clauses: condition.clauses,
      elseGoto: block.conditionalSkipTarget!,
      nextBlock: block.nextBlock,
    };

    unresolvedNodes.push(ifNode);
  } else if (block.endingJump?.type === 'goto') {
    const gotoInstruction = block.endingJump;
    if (!block.gotoJumpTarget) {
      throw new Error(
        `Can't find goto target ${block.endingJump.jumpAddress} at ${block.startAddress}`,
      );
    }
    unresolvedNodes.push({
      type: 'unresolvedGoto',
      jumpTargetAddress: block.gotoJumpTarget.startAddress,
      address: gotoInstruction.address,
    });
  }

  return unresolvedNodes;
}

function resolveNodes(
  unresolvedNodes: UnresolvedLogicASTNode[],
  currentNodeIndex: number,
  labels: Map<number, LogicLabel>,
  nodeIndex?: Map<number, LogicASTNode>,
): LogicASTNode {
  const workingIndex = nodeIndex ?? new Map<number, LogicASTNode>();
  const currentNode = unresolvedNodes[currentNodeIndex];

  const existingNode = workingIndex.get(currentNode.address);
  if (existingNode) {
    return existingNode;
  }

  if (currentNode.type === 'command') {
    const commandNode: LogicCommandNode = {
      ...currentNode,
      label: labels.get(currentNode.address),
    };
    workingIndex.set(currentNode.address, commandNode);
    if (currentNode.agiCommand.name !== 'return' && currentNodeIndex + 1 < unresolvedNodes.length) {
      commandNode.next = resolveNodes(unresolvedNodes, currentNodeIndex + 1, labels, workingIndex);
    }
    return commandNode;
  }

  if (currentNode.type === 'unresolvedIf') {
    // we're going to transform an AGI assembly conditional into something like:
    //
    // if (conditions) {
    // } else {
    //   goto(SkipTarget);
    // }
    //
    // which we can then optimize in later passes using control flow analysis
    const ifNode: LogicIfNode = {
      type: 'if',
      clauses: currentNode.clauses,
      then: [],
      else: [],
      label: labels.get(currentNode.address),
    };
    workingIndex.set(currentNode.address, ifNode);
    if (currentNodeIndex + 1 < unresolvedNodes.length) {
      ifNode.next = resolveNodes(unresolvedNodes, currentNodeIndex + 1, labels, workingIndex);
    }

    // insert a virtual goto at the end of the code for the skip target
    const gotoNodeAddress = (max(unresolvedNodes.map((node) => node.address)) ?? -1) + 1;
    const gotoNodeIndex = unresolvedNodes.length;
    unresolvedNodes.push({
      type: 'unresolvedGoto',
      address: gotoNodeAddress,
      jumpTargetAddress: currentNode.elseGoto.startAddress,
    });
    ifNode.else = [resolveNodes(unresolvedNodes, gotoNodeIndex, labels, nodeIndex)];

    return ifNode;
  }

  if (currentNode.type === 'unresolvedGoto') {
    let target = workingIndex.get(currentNode.jumpTargetAddress);

    if (!target) {
      const targetIndex = unresolvedNodes.findIndex(
        (unresolvedNode) => unresolvedNode.address === currentNode.jumpTargetAddress,
      );
      if (targetIndex === -1) {
        throw new Error(
          `Invalid jump to ${currentNode.jumpTargetAddress} at ${currentNode.address}`,
        );
      }
      target = resolveNodes(unresolvedNodes, targetIndex, labels, workingIndex);
    }

    const gotoNode: LogicGotoNode = {
      type: 'goto',
      jumpTarget: target,
      label: labels.get(currentNode.address),
    };
    workingIndex.set(currentNode.address, gotoNode);
    return gotoNode;
  }

  assertNever(currentNode);
}

export function decompileInstructions(instructions: LogicInstruction[]): LogicASTNode {
  const labels = new Map<number, LogicLabel>(
    generateLabels(instructions).map((label) => [label.address, label]),
  );
  const blocks = buildBasicBlocks(instructions);
  const unresolvedNodes = flatMap(blocks, (block) => decompileBlock(block));
  const rootNode = resolveNodes(unresolvedNodes, 0, labels);
  return rootNode;
}
