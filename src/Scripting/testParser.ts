import { readFileSync } from 'fs';
import { generateLogicScript } from './LogicScriptGenerator';
import { parseLogicScript, SyntaxError } from './LogicScriptParser';

const inputFilePath = 'extracted/kq1/logic/103.agilogic';
const input = readFileSync(inputFilePath, 'utf-8');
try {
  const program = parseLogicScript(input);
  console.log(generateLogicScript(program));
} catch (error) {
  if (error instanceof SyntaxError) {
    console.error(error.message);
    console.log(`${inputFilePath}:${error.location.start.line}:${error.location.start.column}`);
    console.log(input.slice(error.location.start.offset, error.location.end.offset));
  } else {
    throw error;
  }
}
