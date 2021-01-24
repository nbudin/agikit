import assertNever from 'assert-never';
import { flatMap } from 'lodash';
import { LogicConditionClause, LogicInstruction } from '../Types/Logic';

type AddressPlaceholder = {
  instruction: LogicInstruction;
  jumpTarget: LogicInstruction;
};

export class LogicAssembler {
  instructions: LogicInstruction[];
  instructionsByDeclaredAddress: Map<number, LogicInstruction>;
  addressPlaceholders: AddressPlaceholder[];

  constructor(instructions: LogicInstruction[]) {
    this.instructions = instructions;
    this.instructionsByDeclaredAddress = new Map(
      instructions.map((instruction) => [instruction.address, instruction]),
    );
    this.addressPlaceholders = [];
  }

  assemble(): Buffer {
    const byteCodeByInstruction = new Map<LogicInstruction, number[]>();
    const instructionAddresses = new Map<LogicInstruction, number>();
    let address = 0;

    this.instructions.forEach((instruction) => {
      const byteCode = this.assembleInstruction(instruction);
      byteCodeByInstruction.set(instruction, byteCode);
      instructionAddresses.set(instruction, address);
      address += byteCode.length;
    });

    this.addressPlaceholders.forEach((placeholder) => {
      const byteCode = byteCodeByInstruction.get(placeholder.instruction);
      const instructionAddress = instructionAddresses.get(placeholder.instruction);
      const targetAddress = instructionAddresses.get(placeholder.jumpTarget);

      if (byteCode == null) {
        throw new Error('Bytecode not found');
      }

      if (instructionAddress == null) {
        throw new Error('Instruction address not found');
      }

      if (targetAddress == null) {
        throw new Error('Target address is not found');
      }

      const offset = targetAddress - (instructionAddress + byteCode.length);
      const offsetBuffer = Buffer.alloc(2);
      offsetBuffer.writeInt16LE(offset);
      const offsetBytes = [...offsetBuffer];
      byteCodeByInstruction.set(
        placeholder.instruction,
        byteCode.slice(0, byteCode.length - 2).concat(offsetBytes),
      );
    });

    return Buffer.from(
      flatMap(this.instructions, (instruction) => {
        const byteCode = byteCodeByInstruction.get(instruction);
        if (byteCode == null) {
          throw new Error('Bytecode not found');
        }

        return byteCode;
      }),
    );
  }

  private assembleInstruction(instruction: LogicInstruction): number[] {
    if (instruction.type === 'command') {
      return [instruction.agiCommand.opcode, ...instruction.args];
    }

    if (instruction.type === 'goto') {
      const jumpTarget = this.instructionsByDeclaredAddress.get(instruction.jumpAddress);
      if (!jumpTarget) {
        throw new Error(`Invalid jump to ${instruction.jumpAddress}`);
      }
      this.addressPlaceholders.push({
        instruction,
        jumpTarget,
      });

      return [0xfe, 0x00, 0x00];
    }

    if (instruction.type === 'condition') {
      const jumpTarget = this.instructionsByDeclaredAddress.get(instruction.skipAddress);
      if (!jumpTarget) {
        throw new Error(`Invalid conditional skip to ${instruction.skipAddress}`);
      }
      this.addressPlaceholders.push({
        instruction,
        jumpTarget,
      });

      return [
        0xff,
        ...flatMap(instruction.clauses, (clause) => this.assembleClause(clause)),
        0xff,
        0x00,
        0x00,
      ];
    }

    assertNever(instruction);
  }

  private assembleClause(clause: LogicConditionClause): number[] {
    if (clause.type === 'test') {
      const negateByte = clause.negate ? [0xfd] : [];
      if (clause.testCommand.varArgs) {
        return [
          ...negateByte,
          clause.testCommand.opcode,
          clause.args.length,
          ...flatMap(clause.args, (value) => {
            const argBuffer = Buffer.alloc(2);
            argBuffer.writeUInt16LE(value);
            return [...argBuffer];
          }),
        ];
      } else {
        return [...negateByte, clause.testCommand.opcode, ...clause.args];
      }
    }

    if (clause.type === 'or') {
      return [0xfc, ...flatMap(clause.orTests, (test) => this.assembleClause(test))];
    }

    assertNever(clause);
  }
}
