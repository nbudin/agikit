import assertNever from 'assert-never';
import { flatMap, max } from 'lodash';
import {
  LogicScriptArgument,
  LogicScriptArgumentList,
  LogicScriptBooleanExpression,
  LogicScriptProgram,
  LogicScriptStatement,
} from './LogicScriptParserTypes';

export function generateLogicScriptForArgument(argument: LogicScriptArgument): string {
  if (argument.type === 'Identifier') {
    return argument.name;
  } else if (argument.type === 'Literal') {
    return JSON.stringify(argument.value);
  }

  assertNever(argument);
}

export function generateLogicScriptForArgumentList(argumentList: LogicScriptArgumentList): string {
  const args = argumentList.map((argument) => generateLogicScriptForArgument(argument));

  return args.join(', ');
}

export function generateLogicScriptForBooleanExpression(
  expression: LogicScriptBooleanExpression,
): string {
  const generateSubExpression = (subExpression: LogicScriptBooleanExpression) => {
    if (subExpression.type === 'TestCall' || subExpression.type === 'NotExpression') {
      return generateLogicScriptForBooleanExpression(subExpression);
    }

    return `(${generateLogicScriptForBooleanExpression(subExpression)})`;
  };

  if (expression.type === 'TestCall') {
    if (expression.testName === 'isset' && expression.argumentList.length === 1) {
      return generateLogicScriptForArgument(expression.argumentList[0]);
    }
    return `${expression.testName}(${generateLogicScriptForArgumentList(expression.argumentList)})`;
  } else if (expression.type === 'AndExpression') {
    return expression.clauses.map(generateSubExpression).join(' && ');
  } else if (expression.type === 'OrExpression') {
    return expression.clauses.map(generateSubExpression).join(' || ');
  } else if (expression.type === 'NotExpression') {
    return `!${generateSubExpression(expression.expression)}`;
  } else if (expression.type === 'BooleanBinaryOperation') {
    return `${generateLogicScriptForArgumentList([expression.left])} ${
      expression.operator
    } ${generateLogicScriptForArgumentList([expression.right])}`;
  } else if (expression.type === 'Identifier') {
    return expression.name;
  }

  assertNever(expression);
}

export function generateLogicScriptForStatement(
  statement: LogicScriptStatement,
  indent: number,
): string {
  const indentSpaces = ' '.repeat(indent);

  if (statement.type === 'Comment') {
    return `${indentSpaces}//${statement.comment}\n`;
  } else if (statement.type === 'Label') {
    const labelIndent = max([indent - 2, 0]);
    return `\n${' '.repeat(labelIndent ?? 0)}${statement.label}:\n`;
  } else if (statement.type === 'MessageDirective') {
    return `#message ${statement.number} ${JSON.stringify(statement.message)}\n`;
  } else if (statement.type === 'CommandCall') {
    return `${indentSpaces}${statement.commandName}(${generateLogicScriptForArgumentList(
      statement.argumentList,
    )});\n`;
  } else if (statement.type === 'IfStatement') {
    const [thenLines, elseLines] = [statement.thenStatements, statement.elseStatements].map(
      (branch) => {
        const branchLines = flatMap(branch, (branchStatement) =>
          generateLogicScriptForStatement(branchStatement, 2)
            .trimRight()
            .split('\n')
            .map((line) => `${line}\n`),
        );
        if (branchLines.length > 0 && branchLines[0] === '\n') {
          return branchLines.slice(1);
        }
        return branchLines;
      },
    );
    const lines = [
      `if (${generateLogicScriptForBooleanExpression(statement.conditions)}) {\n`,
      ...thenLines,
      ...(elseLines.length > 0 ? [`} else {\n`, ...elseLines] : []),
      '}\n',
    ];
    return '\n' + lines.map((line) => `${indentSpaces}${line}`).join('');
  } else if (statement.type === 'UnaryOperationStatement') {
    return `${indentSpaces}${statement.identifier.name}${statement.operation};\n`;
  } else if (statement.type === 'ValueAssignmentStatement') {
    return `${indentSpaces}${statement.assignee.name} = ${generateLogicScriptForArgument(
      statement.value,
    )};\n`;
  } else if (statement.type === 'ArithmeticAssignmentStatement') {
    return `${indentSpaces}${statement.assignee.name} ${
      statement.operator
    }= ${generateLogicScriptForArgument(statement.value)};`;
  } else if (statement.type === 'LeftIndirectAssignmentStatement') {
    return `${indentSpaces}*${statement.assigneePointer.name} = ${generateLogicScriptForArgument(
      statement.value,
    )};\n`;
  } else if (statement.type === 'RightIndirectAssignmentStatement') {
    return `${indentSpaces}${statement.assignee.name} = *${statement.valuePointer.name};\n`;
  }

  assertNever(statement);
}

export function generateLogicScript(program: LogicScriptProgram<LogicScriptStatement>): string {
  return program
    .map((statement) => generateLogicScriptForStatement(statement, 0))
    .join('')
    .trimLeft();
}
