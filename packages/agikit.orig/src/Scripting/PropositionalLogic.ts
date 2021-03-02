import assertNever from 'assert-never';
import { flatMap } from 'lodash';
import { types } from 'util';
import { AGICommandArgType } from '../Types/AGICommands';
import { IdentifierMapping } from './LogicScriptASTGenerator';
import {
  LogicScriptBooleanBinaryOperation,
  LogicScriptBooleanExpression,
  LogicScriptNotExpression,
  LogicScriptOrExpression,
  LogicScriptTestCall,
} from './LogicScriptParserTypes';

interface BinaryOr<Left extends BinaryExpression, Right extends BinaryExpression> {
  type: 'BinaryOr';
  clauses: [Left, Right];
}

interface BinaryAnd<Left extends BinaryExpression, Right extends BinaryExpression> {
  type: 'BinaryAnd';
  clauses: [Left, Right];
}

interface BinaryNot<T extends BinaryExpression> {
  type: 'BinaryNot';
  expression: T;
}

type BinaryExpression =
  | BinaryOr<BinaryExpression, BinaryExpression>
  | BinaryAnd<BinaryExpression, BinaryExpression>
  | BinaryNot<BinaryExpression>
  | LogicScriptTestCall;

type EitherWayOr<Left extends BinaryExpression, Right extends BinaryExpression> =
  | BinaryOr<Left, Right>
  | BinaryOr<Right, Left>;

type EitherWayAnd<Left extends BinaryExpression, Right extends BinaryExpression> =
  | BinaryAnd<Left, Right>
  | BinaryAnd<Right, Left>;

type IrreducibleBinaryExpression = LogicScriptTestCall | BinaryNot<LogicScriptTestCall>;

type SimpleDistributableOr = EitherWayOr<
  BinaryAnd<IrreducibleBinaryExpression, IrreducibleBinaryExpression>,
  IrreducibleBinaryExpression
>;

type SimpleConcatenatableOr = EitherWayOr<
  BinaryOr<IrreducibleBinaryExpression, IrreducibleBinaryExpression>,
  IrreducibleBinaryExpression
>;

type SimpleConcatenatableAnd = EitherWayAnd<
  BinaryAnd<IrreducibleBinaryExpression, IrreducibleBinaryExpression>,
  IrreducibleBinaryExpression
>;

type BinaryOrContainingAnd = EitherWayOr<
  BinaryAnd<BinaryExpression, BinaryExpression>,
  BinaryExpression
>;

type FindAndClause<T extends BinaryOrContainingAnd> = T extends BinaryOr<
  BinaryAnd<BinaryExpression, BinaryExpression>,
  BinaryExpression
>
  ? T['clauses'][0]
  : T['clauses'][1];
type FindNonAndClause<T extends BinaryOrContainingAnd> = T extends BinaryOr<
  BinaryAnd<BinaryExpression, BinaryExpression>,
  BinaryExpression
>
  ? T['clauses'][1]
  : T['clauses'][0];

export type StrictNotExpression = {
  type: 'StrictNotExpression';
  expression: LogicScriptTestCall;
};

export type StrictOrExpression = {
  type: 'StrictOrExpression';
  clauses: (StrictNotExpression | LogicScriptTestCall)[];
};

export type StrictAndExpression = {
  type: 'StrictAndExpression';
  clauses: (StrictOrExpression | StrictNotExpression | LogicScriptTestCall)[];
};

export type StrictBooleanExpression =
  | StrictNotExpression
  | StrictOrExpression
  | StrictAndExpression
  | LogicScriptTestCall;

function flipBinaryOperator(
  operator: LogicScriptBooleanBinaryOperation['operator'],
): LogicScriptBooleanBinaryOperation['operator'] {
  if (operator === '==' || operator === '!=') {
    return operator;
  }

  if (operator === '<') {
    return '>';
  }

  if (operator === '>') {
    return '<';
  }

  if (operator === '<=') {
    return '>=';
  }

  if (operator === '>=') {
    return '<=';
  }

  assertNever(operator);
}

function logicScriptBooleanOperationToTestCallForm(
  operation: LogicScriptBooleanBinaryOperation,
  identifiers: Map<string, IdentifierMapping>,
): LogicScriptTestCall | LogicScriptOrExpression | LogicScriptNotExpression {
  const { operator, left, right } = operation;
  if (left.type !== 'Identifier') {
    if (right.type !== 'Identifier') {
      throw new Error('At least one side of a boolean operation must be an identifier');
    }

    return logicScriptBooleanOperationToTestCallForm(
      { ...operation, operator: flipBinaryOperator(operator), left: right, right: left },
      identifiers,
    );
  }

  const leftMapping = identifiers.get(left.name);
  if (!leftMapping) {
    throw new Error(`Unknown identifier: ${left.name}`);
  }
  if (leftMapping.type !== AGICommandArgType.Variable) {
    throw new Error(
      `${operator} operations can only use numbers or variables, but ${left.name} is a ${leftMapping.type}`,
    );
  }

  let typeSuffix: string;

  if (right.type === 'Literal') {
    const { value } = right;
    if (typeof value !== 'number') {
      throw new Error(
        `${operator} operations can only use numbers or variables, but "${value}" is a ${typeof value}`,
      );
    }
    typeSuffix = 'n';
  } else {
    const rightMapping = identifiers.get(right.name);
    if (!rightMapping) {
      throw new Error(`Unknown identifier: ${right.name}`);
    }
    if (rightMapping.type !== AGICommandArgType.Variable) {
      throw new Error(
        `${operator} operations can only use numbers or variables, but ${right.name} is a ${rightMapping.type}`,
      );
    }
    typeSuffix = 'v';
  }

  if (operator === '==') {
    return { type: 'TestCall', argumentList: [left, right], testName: `equal${typeSuffix}` };
  }

  if (operator === '!=') {
    return {
      type: 'NotExpression',
      expression: { type: 'TestCall', argumentList: [left, right], testName: `equal${typeSuffix}` },
    };
  }

  if (operator === '<') {
    return { type: 'TestCall', argumentList: [left, right], testName: `less${typeSuffix}` };
  }

  if (operator === '<=') {
    return {
      type: 'OrExpression',
      clauses: [
        { type: 'TestCall', argumentList: [left, right], testName: `less${typeSuffix}` },
        { type: 'TestCall', argumentList: [left, right], testName: `equal${typeSuffix}` },
      ],
    };
  }

  if (operator === '>') {
    return { type: 'TestCall', argumentList: [left, right], testName: `greater${typeSuffix}` };
  }

  if (operator === '>=') {
    return {
      type: 'OrExpression',
      clauses: [
        { type: 'TestCall', argumentList: [left, right], testName: `less${typeSuffix}` },
        { type: 'TestCall', argumentList: [left, right], testName: `equal${typeSuffix}` },
      ],
    };
  }

  assertNever(operator);
}

function logicScriptBooleanToBinaryExpression(
  boolean: LogicScriptBooleanExpression,
  identifiers: Map<string, IdentifierMapping>,
): BinaryExpression {
  if (boolean.type === 'TestCall') {
    return boolean;
  }

  if (boolean.type === 'BooleanBinaryOperation') {
    return logicScriptBooleanToBinaryExpression(
      logicScriptBooleanOperationToTestCallForm(boolean, identifiers),
      identifiers,
    );
  }

  if (boolean.type === 'NotExpression') {
    return {
      type: 'BinaryNot',
      expression: logicScriptBooleanToBinaryExpression(boolean.expression, identifiers),
    };
  }

  if (boolean.clauses.length === 1) {
    return logicScriptBooleanToBinaryExpression(boolean.clauses[0], identifiers);
  }

  if (boolean.type === 'AndExpression') {
    return {
      type: 'BinaryAnd',
      clauses: [
        logicScriptBooleanToBinaryExpression(boolean.clauses[0], identifiers),
        logicScriptBooleanToBinaryExpression(
          {
            ...boolean,
            clauses: boolean.clauses.slice(1),
          },
          identifiers,
        ),
      ],
    };
  }

  if (boolean.type === 'OrExpression') {
    return {
      type: 'BinaryOr',
      clauses: [
        logicScriptBooleanToBinaryExpression(boolean.clauses[0], identifiers),
        logicScriptBooleanToBinaryExpression(
          {
            ...boolean,
            clauses: boolean.clauses.slice(1),
          },
          identifiers,
        ),
      ],
    };
  }

  assertNever(boolean);
}

function isIrreducibleExpression(
  expression: BinaryExpression,
): expression is IrreducibleBinaryExpression {
  return (
    expression.type === 'TestCall' ||
    (expression.type === 'BinaryNot' && expression.expression.type === 'TestCall')
  );
}

function isSimpleDistributableOr(
  expression: BinaryExpression,
): expression is SimpleDistributableOr {
  if (expression.type !== 'BinaryOr') {
    return false;
  }

  const [a, b] = expression.clauses;
  if (a.type === 'BinaryAnd') {
    return [...a.clauses, b].every(isIrreducibleExpression);
  } else if (b.type === 'BinaryAnd') {
    return [...b.clauses, a].every(isIrreducibleExpression);
  } else {
    return false;
  }
}

function isSimpleConcatenatableOr(
  expression: BinaryExpression,
): expression is SimpleConcatenatableOr {
  if (expression.type !== 'BinaryOr') {
    return false;
  }

  const [a, b] = expression.clauses;
  if (a.type === 'BinaryOr') {
    return [...a.clauses, b].every(isIrreducibleExpression);
  } else if (b.type === 'BinaryOr') {
    return [...b.clauses, a].every(isIrreducibleExpression);
  } else {
    return false;
  }
}

function isSimpleConcatenatableAnd(
  expression: BinaryExpression,
): expression is SimpleConcatenatableAnd {
  if (expression.type !== 'BinaryAnd') {
    return false;
  }

  const [a, b] = expression.clauses;
  if (a.type === 'BinaryAnd') {
    return [...a.clauses, b].every(isIrreducibleExpression);
  } else if (b.type === 'BinaryAnd') {
    return [...b.clauses, a].every(isIrreducibleExpression);
  } else {
    return false;
  }
}

function findAndClause<T extends BinaryOrContainingAnd>(expression: T): FindAndClause<T> {
  const [a, b] = expression.clauses;
  if (a.type === 'BinaryAnd') {
    return a as FindAndClause<T>;
  } else {
    return b as FindAndClause<T>;
  }
}

function findNonAndClause<T extends BinaryOrContainingAnd>(expression: T): FindNonAndClause<T> {
  const [a, b] = expression.clauses;
  if (a.type === 'BinaryAnd') {
    return b as FindNonAndClause<T>;
  } else {
    return a as FindNonAndClause<T>;
  }
}

function distributeBinaryOrOverAnd<T extends BinaryOrContainingAnd>(
  expression: T,
): BinaryAnd<
  BinaryOr<FindAndClause<T>['clauses'][0], FindNonAndClause<T>>,
  BinaryOr<FindAndClause<T>['clauses'][1], FindNonAndClause<T>>
> {
  const andClause = findAndClause(expression);
  const otherClause = findNonAndClause(expression);

  return {
    type: 'BinaryAnd',
    clauses: [
      {
        type: 'BinaryOr',
        clauses: [andClause.clauses[0], otherClause],
      },
      {
        type: 'BinaryOr',
        clauses: [andClause.clauses[1], otherClause],
      },
    ],
  };
}

function distributeStrictOrOverAnd(
  andClause: StrictAndExpression,
  nonAndClause: Exclude<StrictBooleanExpression, StrictAndExpression>,
): StrictAndExpression {
  return {
    type: 'StrictAndExpression',
    clauses: andClause.clauses.map((subClause) => concatenateComplexOr([subClause, nonAndClause])),
  };
}

function concatenateOr(expression: SimpleConcatenatableOr): StrictOrExpression {
  const orClause = (expression.clauses[0].type === 'BinaryOr'
    ? expression.clauses[0]
    : expression.clauses[1]) as BinaryOr<IrreducibleBinaryExpression, IrreducibleBinaryExpression>;
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const irreducibleClause: IrreducibleBinaryExpression = expression.clauses.find(
    isIrreducibleExpression,
  )!;

  return {
    type: 'StrictOrExpression',
    clauses: [...orClause.clauses, irreducibleClause].map(simplifyIrreducibleExpression),
  };
}

function concatenateAnd(expression: SimpleConcatenatableAnd): StrictAndExpression {
  const andClause = (expression.clauses[0].type === 'BinaryAnd'
    ? expression.clauses[0]
    : expression.clauses[1]) as BinaryAnd<IrreducibleBinaryExpression, IrreducibleBinaryExpression>;
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const irreducibleClause: IrreducibleBinaryExpression = expression.clauses.find(
    isIrreducibleExpression,
  )!;

  return {
    type: 'StrictAndExpression',
    clauses: [...andClause.clauses, irreducibleClause].map(simplifyIrreducibleExpression),
  };
}

function concatenateComplexAnd(clauses: StrictBooleanExpression[]): StrictAndExpression {
  const concatenatedClauses: StrictAndExpression['clauses'] = flatMap(clauses, (clause) => {
    if (clause.type === 'StrictAndExpression') {
      return concatenateComplexAnd(clause.clauses).clauses;
    }

    return clause;
  });

  return {
    type: 'StrictAndExpression',
    clauses: concatenatedClauses,
  };
}

function concatenateComplexOr(
  clauses: (StrictOrExpression | StrictNotExpression | LogicScriptTestCall)[],
): StrictOrExpression {
  const concatenatedClauses: StrictOrExpression['clauses'] = flatMap(clauses, (clause) => {
    if (clause.type === 'StrictOrExpression') {
      return concatenateComplexOr(clause.clauses).clauses;
    }

    return clause;
  });

  return {
    type: 'StrictOrExpression',
    clauses: concatenatedClauses,
  };
}

function simplifyIrreducibleExpression(
  expression: IrreducibleBinaryExpression,
): StrictNotExpression | LogicScriptTestCall {
  if (expression.type === 'BinaryNot') {
    return {
      type: 'StrictNotExpression',
      expression: expression.expression,
    };
  }

  return expression;
}

function simplifyBinaryExpression(expression: BinaryExpression): StrictBooleanExpression {
  if (isIrreducibleExpression(expression)) {
    return simplifyIrreducibleExpression(expression);
  }

  if (expression.type === 'BinaryOr') {
    const [a, b] = expression.clauses;
    if (isIrreducibleExpression(a) && isIrreducibleExpression(b)) {
      return {
        type: 'StrictOrExpression',
        clauses: [simplifyIrreducibleExpression(a), simplifyIrreducibleExpression(b)],
      };
    }

    if (isSimpleDistributableOr(expression)) {
      const distributed = distributeBinaryOrOverAnd(expression);
      return {
        type: 'StrictAndExpression',
        clauses: distributed.clauses.map((clause) => ({
          type: 'StrictOrExpression',
          clauses: clause.clauses.map(simplifyIrreducibleExpression),
        })),
      };
    }

    if (isSimpleConcatenatableOr(expression)) {
      return concatenateOr(expression);
    }

    const simplifiedClauses: [StrictBooleanExpression, StrictBooleanExpression] = [
      simplifyBinaryExpression(a),
      simplifyBinaryExpression(b),
    ];
    const orClauses = simplifiedClauses.filter(
      (clause): clause is StrictOrExpression['clauses'][number] | StrictOrExpression =>
        clause.type === 'StrictNotExpression' ||
        clause.type === 'TestCall' ||
        clause.type === 'StrictOrExpression',
    );
    const andClauses = simplifiedClauses.filter(
      (clause): clause is StrictAndExpression => clause.type === 'StrictAndExpression',
    );

    if (andClauses.length === 0) {
      return concatenateComplexOr(orClauses);
    } else if (orClauses.length === 0) {
      return concatenateComplexAnd(andClauses);
    } else {
      return distributeStrictOrOverAnd(andClauses[0], orClauses[0]);
    }
  }

  if (expression.type === 'BinaryAnd') {
    const [a, b] = expression.clauses;
    if (isIrreducibleExpression(a) && isIrreducibleExpression(b)) {
      return {
        type: 'StrictAndExpression',
        clauses: [simplifyIrreducibleExpression(a), simplifyIrreducibleExpression(b)],
      };
    }

    if (isSimpleConcatenatableAnd(expression)) {
      return concatenateAnd(expression);
    }

    const simplifiedClauses = expression.clauses.map((clause) => simplifyBinaryExpression(clause));

    return concatenateComplexAnd(simplifiedClauses);
  }

  if (expression.type === 'BinaryNot') {
    const innerExpression = expression.expression;
    if (innerExpression.type === 'BinaryNot') {
      return simplifyBinaryExpression(innerExpression.expression);
    }

    if (innerExpression.type === 'BinaryAnd') {
      return simplifyBinaryExpression({
        type: 'BinaryOr',
        clauses: [
          {
            type: 'BinaryNot',
            expression: innerExpression.clauses[0],
          },
          {
            type: 'BinaryNot',
            expression: innerExpression.clauses[1],
          },
        ],
      });
    }

    if (innerExpression.type === 'BinaryOr') {
      return simplifyBinaryExpression({
        type: 'BinaryAnd',
        clauses: [
          {
            type: 'BinaryNot',
            expression: innerExpression.clauses[0],
          },
          {
            type: 'BinaryNot',
            expression: innerExpression.clauses[1],
          },
        ],
      });
    }

    // We never actually get here, but this helps the type checker
    if (innerExpression.type === 'TestCall') {
      return simplifyIrreducibleExpression(expression as BinaryNot<LogicScriptTestCall>);
    }

    assertNever(innerExpression);
  }

  assertNever(expression);
}

export function simplifyLogicScriptExpression(
  expression: LogicScriptBooleanExpression,
  identifiers: Map<string, IdentifierMapping>,
): StrictBooleanExpression {
  return simplifyBinaryExpression(logicScriptBooleanToBinaryExpression(expression, identifiers));
}
