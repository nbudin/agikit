import { LogicScriptProgram } from './LogicScriptParserTypes';
import { parse, SyntaxError } from './LogicScriptParser.generated';

export function parseLogicScript(source: string): LogicScriptProgram {
  return parse(source);
}

export { SyntaxError };
