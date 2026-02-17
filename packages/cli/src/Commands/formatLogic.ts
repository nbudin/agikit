import { formatLogicScript as agikitFormat } from '@agikit/core';
import { readFileSync } from 'fs';

export function formatLogicScript(inputFilePath: string): void {
  const input = readFileSync(inputFilePath, 'utf-8');
  console.log(agikitFormat(input));
}
