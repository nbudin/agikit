import { readFileSync } from 'fs';
import { optimizeAST } from './Extract/Logic/ASTOptimization';
import { generateLogicAsm } from './Extract/Logic/CodeGeneration';
import { readWordsTok } from './Extract/ReadWordsTok';
import { LogicCompiler } from './Scripting/LogicCompiler';
import { LogicScriptASTGenerator } from './Scripting/LogicScriptASTGenerator';
import { parseLogicScript, SyntaxError } from './Scripting/LogicScriptParser';

const inputFilePath = 'extracted/kq1/logic/103.agilogic';
const input = readFileSync(inputFilePath, 'utf-8');
const wordList = readWordsTok(
  readFileSync("/Users/nbudin/Downloads/King's Quest 1 (AGI, DOS)/WORDS.TOK"),
);

try {
  const parseTree = parseLogicScript(input);
  const lastStatement = parseTree.program[parseTree.program.length - 1];
  if (lastStatement.type !== 'CommandCall' || lastStatement.commandName !== 'return') {
    parseTree.program.push({
      type: 'CommandCall',
      commandName: 'return',
      argumentList: [],
    });
  }
  const astGenerator = new LogicScriptASTGenerator(parseTree, wordList);
  const root = astGenerator.generateASTForLogicScript(parseTree);
  const graph = optimizeAST(root);
  const compiler = new LogicCompiler(graph);
  const { instructions } = compiler.compile();
  console.log(
    generateLogicAsm({ instructions, messages: astGenerator.generateMessageArray() }, wordList),
  );
} catch (error) {
  if (error instanceof SyntaxError) {
    console.error(error.message);
    console.log(`${inputFilePath}:${error.location.start.line}:${error.location.start.column}`);
    console.log(input.slice(error.location.start.offset, error.location.end.offset));
  } else {
    throw error;
  }
}
