import { LogicInstruction } from 'agikit_core';
import { LogicLabel } from './LogicDecompile';

export function generateLabels(
  instructions: LogicInstruction[],
  existingLabels: LogicLabel[] = [],
): LogicLabel[] {
  const targetAddressesWithRefs = new Map<number, Set<LogicInstruction>>();
  instructions.forEach((instruction) => {
    if (instruction.type === 'goto') {
      if (!targetAddressesWithRefs.has(instruction.jumpAddress)) {
        targetAddressesWithRefs.set(instruction.jumpAddress, new Set<LogicInstruction>());
      }
      targetAddressesWithRefs.get(instruction.jumpAddress)?.add(instruction);
    } else if (instruction.type === 'condition') {
      if (!targetAddressesWithRefs.has(instruction.skipAddress)) {
        targetAddressesWithRefs.set(instruction.skipAddress, new Set<LogicInstruction>());
      }
      targetAddressesWithRefs.get(instruction.skipAddress)?.add(instruction);
    }
  });

  existingLabels.forEach((label) => {
    targetAddressesWithRefs.delete(label.address);
  });

  const generatedLabels = [...targetAddressesWithRefs.keys()]
    .sort((a, b) => a - b)
    .map((targetAddress) => ({
      label: `Address${targetAddress}`,
      address: targetAddress,
      references: [...(targetAddressesWithRefs.get(targetAddress) ?? [])],
    }));

  return [...existingLabels, ...generatedLabels];
}
