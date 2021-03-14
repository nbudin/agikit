import { flatMap, map } from 'lodash';
import { AGICommandArgType } from '../Types/AGICommands';
import { LogicScriptIdentifier } from './LogicScriptParserTypes';

export type VariableIdentifierMapping = {
  identifierType: 'variable';
  name: string;
  number: number;
  type: AGICommandArgType;
};

export type ConstantIdentifierMapping = {
  identifierType: 'constant';
  name: string;
  value: string | number;
};

export type IdentifierMapping = VariableIdentifierMapping | ConstantIdentifierMapping;

export const BUILT_IN_IDENTIFIERS = new Map<string, IdentifierMapping>(
  flatMap([...Array(256).keys()], (index) => [
    {
      identifierType: 'variable' as const,
      name: `v${index}`,
      number: index,
      type: AGICommandArgType.Variable,
    },
    {
      identifierType: 'variable' as const,
      name: `f${index}`,
      number: index,
      type: AGICommandArgType.Flag,
    },
    {
      identifierType: 'variable' as const,
      name: `o${index}`,
      number: index,
      type: AGICommandArgType.Object,
    },
    {
      identifierType: 'variable' as const,
      name: `c${index}`,
      number: index,
      type: AGICommandArgType.CtrlCode,
    },
    {
      identifierType: 'variable' as const,
      name: `i${index}`,
      number: index,
      type: AGICommandArgType.Item,
    },
    {
      identifierType: 'variable' as const,
      name: `s${index}`,
      number: index,
      type: AGICommandArgType.String,
    },
    {
      identifierType: 'variable' as const,
      name: `m${index}`,
      number: index,
      type: AGICommandArgType.Message,
    },
  ]).map((identifierMapping) => [identifierMapping.name, identifierMapping]),
);

export function resolveIdentifierMapping(
  identifier: string,
  identifierMappings: Map<string, IdentifierMapping>,
): IdentifierMapping {
  const mapping = identifierMappings.get(identifier);
  if (mapping == null) {
    throw new Error(`"${identifier}" is not defined`);
  }

  return mapping;
}
