import { readFileSync } from 'fs';
import { generateLogicScript } from 'agikit-core/dist/Scripting/LogicScriptGenerator';
import {
  parseLogicScript,
  SyntaxErrorWithFilePath,
} from 'agikit-core/dist/Scripting/LogicScriptParser';

export function formatLogicScript(inputFilePath: string): void {
  const input = readFileSync(inputFilePath, 'utf-8');

  try {
    const program = parseLogicScript(input, inputFilePath);
    console.log(generateLogicScript(program.program));
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
