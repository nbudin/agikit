import {
  parseLogicScriptRaw,
  SyntaxErrorWithFilePath,
} from "agikit-core/dist/Scripting/LogicScriptParser";
import { SyntaxError } from "agikit-core/dist/Scripting/LogicScriptParser.generated";
import {
  LogicScriptArgument,
  LogicScriptBooleanExpression,
  LogicScriptStatement,
  PegJSLocationRange,
} from "agikit-core/dist/Scripting/LogicScriptParserTypes";
import * as vscode from "vscode";

class LogicSemanticTokensProvider
  implements vscode.DocumentSemanticTokensProvider {
  legend: vscode.SemanticTokensLegend;

  constructor() {
    const tokenTypes = [
      "function",
      "variable",
      "comment",
      "string",
      "number",
      "label",
      "directive",
      "keyword",
      "invalid",
    ];
    const tokenModifiers = ["readonly", "pointer"];
    this.legend = new vscode.SemanticTokensLegend(tokenTypes, tokenModifiers);
  }

  provideDocumentSemanticTokens(
    document: vscode.TextDocument
  ): vscode.ProviderResult<vscode.SemanticTokens> {
    let statements: LogicScriptStatement[];
    const tokensBuilder = new vscode.SemanticTokensBuilder(this.legend);

    try {
      statements = parseLogicScriptRaw(document.getText(), document.fileName);
    } catch (error) {
      if (error instanceof SyntaxError) {
        tokensBuilder.push(
          pegLocationToVscodeRange(error.location),
          "invalid",
          []
        );
      }
      return tokensBuilder.build();
    }

    this.provideTokensForStatements(statements, tokensBuilder);
    return tokensBuilder.build();
  }

  private provideTokensForStatements(
    statements: LogicScriptStatement[],
    tokensBuilder: vscode.SemanticTokensBuilder
  ) {
    statements.forEach((statement) => {
      if (!statement.location) {
        return;
      }

      if (statement.type === "Comment") {
        tokensBuilder.push(
          pegLocationToVscodeRange(statement.location),
          "comment",
          []
        );
      } else if (statement.type === "CommandCall") {
        this.provideTokensForArguments(statement.argumentList, tokensBuilder);
        if (statement.commandNameLocation) {
          tokensBuilder.push(
            pegLocationToVscodeRange(statement.commandNameLocation),
            "function",
            []
          );
        }
      } else if (statement.type === "IfStatement") {
        if (statement.ifKeyword.location) {
          tokensBuilder.push(
            pegLocationToVscodeRange(statement.ifKeyword.location),
            "keyword",
            []
          );
        }

        if (statement.elseKeyword?.location) {
          tokensBuilder.push(
            pegLocationToVscodeRange(statement.elseKeyword.location),
            "keyword",
            []
          );
        }

        this.provideTokensForBooleanExpression(
          statement.conditions,
          tokensBuilder
        );

        this.provideTokensForStatements(
          statement.thenStatements,
          tokensBuilder
        );
        this.provideTokensForStatements(
          statement.elseStatements,
          tokensBuilder
        );
      } else if (statement.type === "ValueAssignmentStatement") {
        this.provideTokensForArguments(
          [statement.assignee, statement.value],
          tokensBuilder
        );
      } else if (statement.type === "UnaryOperationStatement") {
        this.provideTokensForArgument(statement.identifier, tokensBuilder);
      } else if (statement.type === "ArithmeticAssignmentStatement") {
        this.provideTokensForArguments(
          [statement.assignee, statement.value],
          tokensBuilder
        );
      } else if (statement.type === "LeftIndirectAssignmentStatement") {
        if (statement.assigneePointer.location) {
          tokensBuilder.push(
            pegLocationToVscodeRange(statement.assigneePointer.location),
            "variable",
            ["pointer"]
          );
        }
        this.provideTokensForArgument(statement.value, tokensBuilder);
      } else if (statement.type === "RightIndirectAssignmentStatement") {
        if (statement.valuePointer.location) {
          tokensBuilder.push(
            pegLocationToVscodeRange(statement.valuePointer.location),
            "variable",
            ["pointer"]
          );
        }
        this.provideTokensForArgument(statement.assignee, tokensBuilder);
      } else if (statement.type === "Label") {
        tokensBuilder.push(
          pegLocationToVscodeRange(statement.location),
          "label",
          []
        );
      } else if (
        statement.type === "DefineDirective" ||
        statement.type === "IncludeDirective" ||
        statement.type === "MessageDirective"
      ) {
        if (statement.keyword.location) {
          tokensBuilder.push(
            pegLocationToVscodeRange(statement.keyword.location),
            "directive",
            []
          );
        }

        if (statement.type === "DefineDirective") {
          this.provideTokensForArguments(
            [statement.identifier, statement.value],
            tokensBuilder
          );
        } else if (statement.type === "IncludeDirective") {
          this.provideTokensForArgument(statement.filename, tokensBuilder);
        } else if (statement.type === "MessageDirective") {
          this.provideTokensForArguments(
            [statement.message, statement.number],
            tokensBuilder
          );
        }
      }
    });
  }

  private provideTokensForArgument(
    arg: LogicScriptArgument,
    tokensBuilder: vscode.SemanticTokensBuilder
  ) {
    if (!arg.location) {
      return;
    }

    if (arg.type === "Identifier") {
      tokensBuilder.push(
        pegLocationToVscodeRange(arg.location),
        "variable",
        []
      );
    }

    if (arg.type === "Literal") {
      tokensBuilder.push(
        pegLocationToVscodeRange(arg.location),
        typeof arg.value === "number" ? "number" : "string",
        []
      );
    }
  }

  private provideTokensForArguments(
    args: LogicScriptArgument[],
    tokensBuilder: vscode.SemanticTokensBuilder
  ) {
    args.forEach((arg) => {
      this.provideTokensForArgument(arg, tokensBuilder);
    });
  }

  private provideTokensForBooleanExpression(
    expression: LogicScriptBooleanExpression,
    tokensBuilder: vscode.SemanticTokensBuilder
  ) {
    if (
      expression.type === "AndExpression" ||
      expression.type === "OrExpression"
    ) {
      expression.clauses.forEach((clause) => {
        this.provideTokensForBooleanExpression(clause, tokensBuilder);
      });
    } else if (expression.type === "NotExpression") {
      this.provideTokensForBooleanExpression(
        expression.expression,
        tokensBuilder
      );
    } else if (expression.type === "TestCall") {
      this.provideTokensForArguments(expression.argumentList, tokensBuilder);
      if (expression.testNameLocation) {
        tokensBuilder.push(
          pegLocationToVscodeRange(expression.testNameLocation),
          "function",
          []
        );
      }
    } else if (expression.type === "BooleanBinaryOperation") {
      this.provideTokensForArguments(
        [expression.left, expression.right],
        tokensBuilder
      );
    } else if (expression.type === "Identifier") {
      this.provideTokensForArgument(expression, tokensBuilder);
    }
  }
}

export default LogicSemanticTokensProvider;
function pegLocationToVscodeRange(location: PegJSLocationRange): vscode.Range {
  const range = new vscode.Range(
    new vscode.Position(location.start.line - 1, location.start.column - 1),
    new vscode.Position(location.end.line - 1, location.end.column - 1)
  );

  if (!range.isSingleLine) {
    // semantic highlighting can only handle ranges on the same line; make this zero-length for now
    // TODO: something less hacky
    return range.with(undefined, range.start);
  }

  return range;
}
