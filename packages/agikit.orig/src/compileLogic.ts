import { readFileSync } from 'fs';
import { optimizeAST } from './Extract/Logic/ASTOptimization';
import { generateLogicAsm } from './Extract/Logic/CodeGeneration';
import { readLogicResource } from './Extract/Logic/ReadLogic';
import { readWordsTok } from './Extract/ReadWordsTok';
import { LogicAssembler } from './Scripting/LogicAssembler';
import { LogicCompiler } from './Scripting/LogicCompiler';
import { LogicScriptASTGenerator } from './Scripting/LogicScriptASTGenerator';
import { parseLogicScript, SyntaxError } from './Scripting/LogicScriptParser';
import { encodeLogic, encodeMessages } from './Scripting/WriteLogic';

const inputFilePath = 'extracted/kq1/logic/103.agilogic';
const input = readFileSync(inputFilePath, 'utf-8');
const wordList = readWordsTok(
  readFileSync("/Users/nbudin/Downloads/King's Quest 1 (AGI, DOS)/WORDS.TOK"),
);

try {
  const parseTree = parseLogicScript(input);
  const lastStatement = [...parseTree.program]
    .reverse()
    .find((statement) => statement.type === 'CommandCall' || statement.type === 'IfStatement');
  if (
    lastStatement &&
    (lastStatement.type !== 'CommandCall' || lastStatement.commandName !== 'return')
  ) {
    parseTree.program.push({
      type: 'CommandCall',
      commandName: 'return',
      argumentList: [],
    });
  }
  const astGenerator = new LogicScriptASTGenerator(parseTree, wordList);
  const root = astGenerator.generateASTForLogicScript(parseTree);
  const graph = optimizeAST(root);
  const compiler = new LogicCompiler(graph, astGenerator.getLabels());
  const { instructions } = compiler.compile();
  const assembler = new LogicAssembler(instructions);
  const logic = encodeLogic(
    assembler.assemble(),
    encodeMessages(astGenerator.generateMessageArray()),
  );

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
