import { readFileSync } from 'fs';
import { compileLogicScript } from './Build/BuildLogic';
import { generateLogicAsm } from './Extract/Logic/CodeGeneration';
import { readLogicResource } from './Extract/Logic/ReadLogic';
import { readWordsTok } from './Extract/ReadWordsTok';
import { SyntaxError } from './Scripting/LogicScriptParser';

const inputFilePath = 'extracted/kq1/logic/103.agilogic';
const input = readFileSync(inputFilePath, 'utf-8');
const wordList = readWordsTok(
  readFileSync("/Users/nbudin/Downloads/King's Quest 1 (AGI, DOS)/WORDS.TOK"),
);

try {
  const logic = compileLogicScript(input, wordList);
  const resource = readLogicResource(logic, { major: 3, minor: 9999 });
  console.log(generateLogicAsm(resource, wordList));
} catch (error) {
  if (error instanceof SyntaxError) {
    console.error(error.message);
    console.log(`${inputFilePath}:${error.location.start.line}:${error.location.start.column}`);
    console.log(input.slice(error.location.start.offset, error.location.end.offset));
  } else {
    throw error;
  }
}
