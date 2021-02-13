import { LogicInstruction } from '../Types/Logic';
import { optimizeAST } from '../Extract/Logic/ASTOptimization';
import { LogicAssembler } from '../Scripting/LogicAssembler';
import { LogicCompiler } from '../Scripting/LogicCompiler';
import { LogicScriptASTGenerator } from '../Scripting/LogicScriptASTGenerator';
import { parseLogicScript } from '../Scripting/LogicScriptParser';
import { encodeLogic, encodeMessages } from '../Scripting/WriteLogic';
import { WordList } from '../Types/WordList';

export function assembleLogic(
  instructions: LogicInstruction[],
  messages: (string | undefined)[],
): Buffer {
  const assembler = new LogicAssembler(instructions);
  const logic = encodeLogic(assembler.assemble(), encodeMessages(messages));
  return logic;
}

export function compileLogicScript(sourceCode: string, wordList: WordList): Buffer {
  const parseTree = parseLogicScript(sourceCode);
  const astGenerator = new LogicScriptASTGenerator(parseTree, wordList);
  const root = astGenerator.generateASTForLogicScript(parseTree);
  const graph = optimizeAST(root);
  const compiler = new LogicCompiler(graph, astGenerator.getLabels());
  const { instructions } = compiler.compile();
  const messages = astGenerator.generateMessageArray();
  const logic = assembleLogic(instructions, messages);
  return logic;
}
