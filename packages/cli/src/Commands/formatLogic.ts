import {
  parseLogicScriptRaw,
  parseLogicScript,
  generateLogicScript,
  SyntaxErrorWithFilePath,
} from '@agikit/core';
import { readFileSync } from 'fs';

export function formatLogicScript(inputFilePath: string): void {
  const input = readFileSync(inputFilePath, 'utf-8');

  try {
    const rawProgram = parseLogicScriptRaw(input, inputFilePath);
    const parseTree = parseLogicScript(rawProgram, inputFilePath);
    console.log(generateLogicScript(parseTree.program));
  } catch (error) {
    if (error instanceof SyntaxErrorWithFilePath) {
      console.error(error.message);
      console.log(`${error.filePath}:${error.location.start.line}:${error.location.start.column}`);
      console.log(input.slice(error.location.start.offset, error.location.end.offset));
    } else {
      throw error;
    }
  }
}
