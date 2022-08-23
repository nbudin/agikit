import { parseLogicScript, parseLogicScriptRaw } from '../LogicScriptParser';
import { LogicCompiler } from '../LogicCompiler';
import { optimizeAST } from '../../Extract/Logic/ASTOptimization';
import { LogicScriptASTGenerator } from '../LogicScriptASTGenerator';
import { WordList } from '../../Types/WordList';
import { ObjectList } from '../../Types/ObjectList';
import { LogicCommand } from '../../Types/Logic';

it('Issue #3: compiles to assignn or assignv depending on the value type, not the token type', () => {
  const testScript = `
  #define LITERAL   55
  #define VARIABLE v55

  v100 = 55;
  v100 = LITERAL;
  v100 = v55;
  v100 = VARIABLE;
  `;

  const program = parseLogicScriptRaw(testScript, 'test.agilogic');
  const parseTree = parseLogicScript(program, 'test.agilogic');
  const astGenerator = new LogicScriptASTGenerator(parseTree, new Map(), {
    maxAnimatedObjects: 0,
    objects: [],
  });
  const root = astGenerator.generateASTForLogicScript();
  const graph = optimizeAST(root);
  const compiler = new LogicCompiler(graph, astGenerator.getLabels());
  const { instructions } = compiler.compile();

  const commands: LogicCommand[] = instructions.filter(
    (instruction): instruction is LogicCommand => instruction.type === 'command',
  );

  expect(commands.map((command) => command.agiCommand.name)).toEqual([
    'assignn',
    'assignn',
    'assignv',
    'assignv',
  ]);

  commands.forEach((command) => {
    expect(command.args).toEqual([100, 55]);
  });
});
