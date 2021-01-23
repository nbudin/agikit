import assertNever from 'assert-never';
import { max } from 'lodash';
import { LogicConditionClause, LogicCommand, LogicInstruction } from '../../Types/Logic';
import { Graph } from '../Graphs';
import { generateLabels } from './LogicDisasm';

export type LogicLabel = {
  address: number;
  label: string;
  references: LogicInstruction[];
};

export type LogicASTNodeMetadata = {
  instructionAddress?: number;
};

export type LogicCommandNode = LogicCommand & {
  id: string;
  label?: LogicLabel;
  next?: LogicASTNode;
  metadata?: LogicASTNodeMetadata;
};

export type LogicIfNode = {
  type: 'if';
  id: string;
  clauses: LogicConditionClause[];
  then?: LogicASTNode;
  else?: LogicASTNode;
  label?: LogicLabel;
  metadata?: LogicASTNodeMetadata;
};

export type LogicGotoNode = {
  type: 'goto';
  id: string;
  jumpTarget: LogicASTNode;
  label?: LogicLabel;
  metadata?: LogicASTNodeMetadata;
};

export type LogicASTNode = LogicIfNode | LogicGotoNode | LogicCommandNode;

type UnresolvedIfNode = {
  type: 'unresolvedIf';
  address: number;
  clauses: LogicConditionClause[];
  elseGotoAddress: number;
};

type UnresolvedGotoNode = {
  type: 'unresolvedGoto';
  address: number;
  jumpTargetAddress: number;
};

type UnresolvedLogicASTNode = LogicCommand | UnresolvedIfNode | UnresolvedGotoNode;

function decompileInstruction(instruction: LogicInstruction): UnresolvedLogicASTNode {
  if (instruction.type === 'command') {
    return instruction;
  }

  if (instruction.type === 'condition') {
    const ifNode: UnresolvedIfNode = {
      type: 'unresolvedIf',
      address: instruction.address,
      clauses: instruction.clauses,
      elseGotoAddress: instruction.skipAddress,
    };
    return ifNode;
  }

  if (instruction.type === 'goto') {
    return {
      type: 'unresolvedGoto',
      jumpTargetAddress: instruction.jumpAddress,
      address: instruction.address,
    };
  }

  assertNever(instruction);
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
      id: currentNode.address.toString(),
      label: labels.get(currentNode.address),
      metadata: {
        instructionAddress: currentNode.address,
      },
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
    //   stuffAfterTheIf
    // } else {
    //   goto(SkipTarget);
    // }
    //
    // which we can then optimize in later passes using control flow analysis
    const ifNode: LogicIfNode = {
      type: 'if',
      id: currentNode.address.toString(),
      clauses: currentNode.clauses,
      label: labels.get(currentNode.address),
      metadata: {
        instructionAddress: currentNode.address,
      },
    };
    workingIndex.set(currentNode.address, ifNode);
    if (currentNodeIndex + 1 < unresolvedNodes.length) {
      ifNode.then = resolveNodes(unresolvedNodes, currentNodeIndex + 1, labels, workingIndex);
    }

    // insert a virtual goto at the end of the code for the skip target
    const gotoNodeAddress = (max(unresolvedNodes.map((node) => node.address)) ?? -1) + 1;
    const gotoNodeIndex = unresolvedNodes.length;
    unresolvedNodes.push({
      type: 'unresolvedGoto',
      address: gotoNodeAddress,
      jumpTargetAddress: currentNode.elseGotoAddress,
    });
    ifNode.else = resolveNodes(unresolvedNodes, gotoNodeIndex, labels, workingIndex);

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
      id: currentNode.address.toString(),
      jumpTarget: target,
      label: labels.get(currentNode.address),
      metadata: {
        instructionAddress: currentNode.address,
      },
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
  const unresolvedNodes = instructions.map((instruction) => decompileInstruction(instruction));
  const rootNode = resolveNodes(unresolvedNodes, 0, labels);
  return rootNode;
}
