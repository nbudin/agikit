import { readFileSync } from 'fs';
import { readWordsTok } from './Extract/ReadWordsTok';
import { LogicScriptASTGenerator } from './Scripting/LogicScriptASTGenerator';
import { parseLogicScript, SyntaxError } from './Scripting/LogicScriptParser';

const inputFilePath = 'extracted/kq1/logic/103.agilogic';
const input = readFileSync(inputFilePath, 'utf-8');
const wordList = readWordsTok(
  readFileSync("/Users/nbudin/Downloads/King's Quest 1 (AGI, DOS)/WORDS.TOK"),
);

try {
  const program = parseLogicScript(input);
  const astGenerator = new LogicScriptASTGenerator(program, wordList);
  console.log(JSON.stringify(astGenerator.generateASTForLogicScript(program), null, 2));
} catch (error) {
  if (error instanceof SyntaxError) {
    console.error(error.message);
    console.log(`${inputFilePath}:${error.location.start.line}:${error.location.start.column}`);
    console.log(input.slice(error.location.start.offset, error.location.end.offset));
  } else {
    throw error;
  }
}
