import {
  LogicCondition,
  LogicConditionClause,
  LogicInstruction,
  LogicOr,
  LogicTest,
} from '../../Types/Logic';
import { getAGICommand, getTestCommand } from '../../Types/AGICommands';
import { AGIVersion } from '../../Types/AGIVersion';
import { LogicLabel } from './LogicDecompile';

export function readInstructions(codeData: Buffer, agiVersion: AGIVersion): LogicInstruction[] {
  const instructions: LogicInstruction[] = [];
  let offset = 0;
  while (offset < codeData.byteLength) {
    const address = offset;
    const opcode = codeData.readUInt8(offset);
    offset += 1;

    if (opcode === 0xff) {
      const clauses: LogicConditionClause[] = [];
      let negateNext = false;
      let disjunction: LogicOr | undefined;

      while (offset < codeData.byteLength) {
        const testOpcode = codeData.readUInt8(offset);
        offset += 1;

        if (testOpcode === 0xff) {
          break;
        } else if (testOpcode === 0xfd) {
          negateNext = true;
        } else if (testOpcode === 0xfc) {
          if (disjunction) {
            clauses.push(disjunction);
            disjunction = undefined;
          } else {
            disjunction = { type: 'or', orTests: [] };
          }
        } else {
          const testCommand = getTestCommand(testOpcode);
          const args: number[] = [];
          if (!testCommand) {
            throw new Error(`Invalid test opcode ${testOpcode} at ${offset}`);
          }

          if (testCommand.varArgs) {
            const argCount = codeData.readUInt8(offset);
            offset += 1;

            for (let argIndex = 0; argIndex < argCount; argIndex += 1) {
              const arg = codeData.readUInt16LE(offset);
              offset += 2;
              args.push(arg);
            }
          } else {
            testCommand.argTypes.forEach(() => {
              args.push(codeData.readUInt8(offset));
              offset += 1;
            });
          }

          const test: LogicTest = { type: 'test', args, negate: negateNext, testCommand };

          if (disjunction) {
            disjunction.orTests.push(test);
          } else {
            clauses.push(test);
          }
          negateNext = false;
        }
      }

      const skipOffset = codeData.readInt16LE(offset);
      offset += 2;

      const condition: LogicCondition = {
        type: 'condition',
        address,
        clauses,
        skipAddress: skipOffset + offset,
      };
      instructions.push({ ...condition, address });
    } else if (opcode === 0xfe) {
      const jumpOffset = codeData.readInt16LE(offset);
      offset += 2;
      instructions.push({ type: 'goto', jumpAddress: jumpOffset + offset, address });
    } else {
      const agiCommand = getAGICommand(opcode, agiVersion);
      if (agiCommand == null) {
        throw new Error(`Invalid opcode ${opcode} at offset ${offset}`);
      }
      const args: number[] = [];
      agiCommand.argTypes.forEach(() => {
        args.push(codeData.readUInt8(offset));
        offset += 1;
      });
      instructions.push({ type: 'command', agiCommand, args, address });
    }
  }

  return instructions;
}

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
