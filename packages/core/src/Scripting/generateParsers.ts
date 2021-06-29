import { readFileSync, writeFileSync } from 'fs';
import pegjs from 'pegjs';
// @ts-expect-error screw it
import tspegjs from 'ts-pegjs';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function generateParser(sourcePath: string, outputPath: string, options: any) {
  const grammar = readFileSync(sourcePath, 'utf-8');
  const parser = pegjs.generate(grammar, {
    output: 'source',
    // @ts-expect-error tspegjs option breaks this
    format: 'commonjs',
    plugins: [tspegjs],
    tspegjs: options,
  });

  writeFileSync(outputPath, parser);
}

generateParser('src/Scripting/LogicScript.pegjs', 'src/Scripting/LogicScriptParser.generated.ts', {
  customHeader: `import { LogicScriptProgram } from './LogicScriptParserTypes';`,
  returnTypes: {
    Program: 'LogicScriptProgram',
  },
});

generateParser('src/Scripting/WordList.pegjs', 'src/Scripting/WordListParser.generated.ts', {});
