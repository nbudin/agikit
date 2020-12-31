import { keyBy } from 'lodash';
import { AGIVersion } from './AGIVersion';
import agiCommandsData from './agiCommands.json';
import testCommandsData from './testCommands.json';

export enum AGICommandArgType {
  Number = 'Number',
  Variable = 'Variable',
  Flag = 'Flag',
  Message = 'Message',
  Object = 'Object',
  Item = 'Item',
  String = 'String',
  Word = 'Word',
  CtrlCode = 'CtrlCode',
}

export type AGICommand = {
  opcode: number;
  name: string;
  argTypes: AGICommandArgType[];
};

export type TestCommand = {
  opcode: number;
  name: string;
  argTypes: AGICommandArgType[];
  varArgs?: true;
};

export const agiCommands = agiCommandsData as AGICommand[];
export const testCommands = testCommandsData as TestCommand[];

const agiCommandsByOpcode = keyBy(agiCommands, (cmd) => cmd.opcode);
const testCommandsByOpcode = keyBy(testCommands, (cmd) => cmd.opcode);

export function getAGICommand(opcode: number, agiVersion: AGIVersion): AGICommand | undefined {
  if (opcode > 177 && (agiVersion.major < 3 || agiVersion.minor <= 2086)) {
    return undefined;
  } else if (opcode > 175 && agiVersion.major === 2 && agiVersion.minor <= 936) {
    return undefined;
  } else if (opcode > 173 && agiVersion.major === 2 && agiVersion.minor <= 917) {
    return undefined;
  } else if (opcode > 169 && agiVersion.major === 2 && agiVersion.minor <= 440) {
    return undefined;
  } else if (opcode > 161 && agiVersion.major === 2 && agiVersion.minor <= 272) {
    return undefined;
  } else if (opcode > 155 && agiVersion.major === 2 && agiVersion.minor <= 89) {
    return undefined;
  }

  if (opcode === 134 && agiVersion.major === 2 && agiVersion.minor <= 89) {
    return { ...agiCommandsByOpcode[opcode], argTypes: [] }; // quit
  }

  if ((opcode === 151 || opcode === 152) && agiVersion.major === 2 && agiVersion.minor < 400) {
    // print.at and print.at.v
    return {
      ...agiCommandsByOpcode[opcode],
      argTypes: agiCommandsByOpcode[opcode].argTypes.slice(0, 2),
    };
  }

  if (opcode === 176 && (agiVersion.major === 2 || agiVersion.minor <= 2086)) {
    // hide.mouse
    return {
      ...agiCommandsByOpcode[opcode],
      argTypes: [AGICommandArgType.Number],
    };
  }

  return agiCommandsByOpcode[opcode];
}

export function getTestCommand(opcode: number): TestCommand | undefined {
  return testCommandsByOpcode[opcode];
}
