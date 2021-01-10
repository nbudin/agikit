import { readFileSync, writeFileSync } from 'fs';
import pegjs from 'pegjs';
// @ts-expect-error screw it
import tspegjs from 'ts-pegjs';

const grammar = readFileSync('src/Scripting/LogicScript.pegjs', 'utf-8');
const parser = pegjs.generate(grammar, {
  output: 'source',
  // @ts-expect-error tspegjs option breaks this
  format: 'commonjs',
  plugins: [tspegjs],
  tspegjs: {
    customHeader: `import { LogicScriptProgram } from './LogicScriptParserTypes';`,
    returnTypes: {
      Program: 'LogicScriptProgram',
    },
  },
});

writeFileSync('src/Scripting/LogicScriptParser.generated.ts', parser);
