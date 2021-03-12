import {
  CompletionItem,
  CompletionItemKind,
  createConnection,
  Definition,
  DefinitionLink,
  Diagnostic,
  DiagnosticSeverity,
  DidChangeConfigurationNotification,
  DocumentLink,
  FileChangeType,
  InitializeParams,
  InitializeResult,
  Location,
  MarkupKind,
  Position,
  ProposedFeatures,
  Range,
  TextDocumentPositionParams,
  TextDocuments,
  TextDocumentSyncKind,
} from "vscode-languageserver/node";
import {
  LogicScriptParseTree,
  parseLogicScriptRaw,
  buildIdentifierMappingForDefineDirective,
} from "agikit-core/dist/Scripting/LogicScriptParser";
import { URI, Utils } from "vscode-uri";
import fs from "fs";
import path from "path";
import { TextDocument } from "vscode-languageserver-textdocument";
import { SyntaxError } from "agikit-core/dist/Scripting/LogicScriptParser.generated";
import { agiCommands } from "agikit-core/dist/Types/AGICommands";
import {
  LogicScriptArgument,
  LogicScriptBooleanExpression,
  LogicScriptDefineDirective,
  LogicScriptIdentifier,
  LogicScriptIncludeDirective,
  LogicScriptProgram,
  LogicScriptStatement,
  PegJSLocationRange,
} from "agikit-core/dist/Scripting/LogicScriptParserTypes";
import {
  BUILT_IN_IDENTIFIERS,
  IdentifierMapping,
} from "agikit-core/dist/Scripting/LogicScriptIdentifierMapping";

const documents: TextDocuments<TextDocument> = new TextDocuments(TextDocument);
const parseTrees = new Map<
  string,
  LogicScriptParseTree<LogicScriptStatement>
>();

type LogicScriptDefine = {
  directive: LogicScriptDefineDirective;
  fileUri: string;
  identifierMapping: IdentifierMapping;
};
type LogicScriptIdentifierWithFileUri = {
  identifier: LogicScriptIdentifier;
  fileUri: string;
};
const definesByName = new Map<string, LogicScriptDefine[]>();
const definesByDocument = new Map<string, LogicScriptDefine[]>();
const identifiersByDocument = new Map<string, LogicScriptIdentifier[]>();
const identifiersByName = new Map<string, LogicScriptIdentifierWithFileUri[]>();
const includeDirectivesByDocument = new Map<
  string,
  LogicScriptIncludeDirective[]
>();
const includedDocumentsByDocument = new Map<string, string[]>();

let connection = createConnection(ProposedFeatures.all);

let hasConfigurationCapability: boolean = false;
let hasWorkspaceFolderCapability: boolean = false;
let hasDiagnosticRelatedInformationCapability: boolean = false;

function listFilesRecursive(dirPath: string): string[] {
  const entries = fs.readdirSync(dirPath);
  const files: string[] = [];

  entries.forEach((entry) => {
    const entryPath = path.resolve(dirPath, entry);
    const stat = fs.statSync(entryPath);
    if (stat.isFile()) {
      files.push(entryPath);
    } else if (stat.isDirectory()) {
      files.push(...listFilesRecursive(entryPath));
    }
  });

  return files;
}

function pegJSLocationRangeToVSCodeRange(location: PegJSLocationRange): Range {
  return {
    start: {
      line: location.start.line - 1,
      character: location.start.column - 1,
    },
    end: { line: location.end.line - 1, character: location.end.column - 1 },
  };
}

connection.onInitialize((params: InitializeParams) => {
  let capabilities = params.capabilities;

  // Does the client support the `workspace/configuration` request?
  // If not, we fall back using global settings.
  hasConfigurationCapability = !!(
    capabilities.workspace && !!capabilities.workspace.configuration
  );
  hasWorkspaceFolderCapability = !!(
    capabilities.workspace && !!capabilities.workspace.workspaceFolders
  );
  hasDiagnosticRelatedInformationCapability = !!(
    capabilities.textDocument &&
    capabilities.textDocument.publishDiagnostics &&
    capabilities.textDocument.publishDiagnostics.relatedInformation
  );

  const result: InitializeResult = {
    capabilities: {
      textDocumentSync: TextDocumentSyncKind.Incremental,
      // Tell the client that this server supports code completion.
      completionProvider: {
        resolveProvider: true,
      },
      documentLinkProvider: {
        resolveProvider: true,
      },
      definitionProvider: {},
      hoverProvider: {},
      referencesProvider: {},
    },
  };
  if (hasWorkspaceFolderCapability) {
    result.capabilities.workspace = {
      workspaceFolders: {
        supported: true,
      },
    };
  }
  return result;
});

connection.onInitialized(() => {
  if (hasConfigurationCapability) {
    // Register for all configuration changes.
    connection.client.register(
      DidChangeConfigurationNotification.type,
      undefined
    );
  }
  if (hasWorkspaceFolderCapability) {
    connection.workspace.onDidChangeWorkspaceFolders((event) => {
      event.added.forEach((folder) => {
        const uri = URI.parse(folder.uri);
        if (uri.scheme === "file") {
          const logicFiles = listFilesRecursive(uri.fsPath).filter((filePath) =>
            filePath.toLowerCase().endsWith(".agilogic")
          );
          logicFiles.forEach((logicFile) =>
            refreshTextDocument(uri, fs.readFileSync(logicFile, "utf-8"))
          );
        }
      });
      connection.console.log("Workspace folder change event received.");
    });
  }
});

// The example settings
interface ExampleSettings {
  maxNumberOfProblems: number;
}

// The global settings, used when the `workspace/configuration` request is not supported by the client.
// Please note that this is not the case when using this server with the client provided in this example
// but could happen with other clients.
const defaultSettings: ExampleSettings = { maxNumberOfProblems: 1000 };
let globalSettings: ExampleSettings = defaultSettings;

// Cache the settings of all open documents
let documentSettings: Map<string, Thenable<ExampleSettings>> = new Map();

connection.onDidChangeConfiguration((change) => {
  if (hasConfigurationCapability) {
    // Reset all cached document settings
    documentSettings.clear();
  } else {
    globalSettings = <ExampleSettings>(
      (change.settings.agikit || defaultSettings)
    );
  }

  // Revalidate all open text documents
  documents.all().forEach((document) => {
    refreshTextDocument(URI.parse(document.uri), document.getText());
  });
});

function getDocumentSettings(resource: string): Thenable<ExampleSettings> {
  if (!hasConfigurationCapability) {
    return Promise.resolve(globalSettings);
  }
  let result = documentSettings.get(resource);
  if (!result) {
    result = connection.workspace.getConfiguration({
      scopeUri: resource,
      section: "agikit",
    });
    documentSettings.set(resource, result);
  }
  return result;
}

// Only keep settings for open documents
documents.onDidClose((e) => {
  documentSettings.delete(e.document.uri);
});

// The content of a text document has changed. This event is emitted
// when the text document first opened or when its content has changed.
documents.onDidChangeContent((change) => {
  refreshTextDocument(
    URI.parse(change.document.uri),
    change.document.getText()
  );
});

function clearDocumentData(uri: URI) {
  const uriString = uri.toString();
  parseTrees.delete(uriString);
  const defines = definesByDocument.get(uriString);
  if (defines) {
    defines.forEach((define) => {
      const defineList = definesByName.get(define.directive.identifier.name);
      if (defineList) {
        definesByName.set(
          define.directive.identifier.name,
          defineList.filter((define) => define.fileUri !== uriString)
        );
      }
    });
  }
  const identifiers = identifiersByDocument.get(uriString);
  if (identifiers) {
    identifiers.forEach((identifier) => {
      const identifierList = identifiersByName.get(identifier.name);
      if (identifierList) {
        identifiersByName.set(
          identifier.name,
          identifierList.filter(
            (identifier) => identifier.fileUri !== uriString
          )
        );
      }
    });
  }
  definesByDocument.delete(uriString);
  includeDirectivesByDocument.delete(uriString);
  includedDocumentsByDocument.delete(uriString);
  identifiersByDocument.delete(uriString);
}

async function refreshTextDocument(uri: URI, contents: string): Promise<void> {
  let settings = await getDocumentSettings(uri.toString());

  let diagnostics: Diagnostic[] = [];
  let statements: LogicScriptProgram<LogicScriptStatement> | undefined;
  try {
    statements = parseLogicScriptRaw(contents, uri.fsPath);
  } catch (error) {
    if (error instanceof SyntaxError) {
      let diagnostic: Diagnostic = {
        severity: DiagnosticSeverity.Error,
        range: pegJSLocationRangeToVSCodeRange(error.location),
        message: error.message,
        source: "agikit",
      };
      diagnostics.push(diagnostic);
    } else {
      throw error;
    }
  }

  if (statements) {
    clearDocumentData(uri);
    const documentDefines: LogicScriptDefine[] = [];
    const documentIncludes: LogicScriptIncludeDirective[] = [];
    const documentIdentifiers: LogicScriptIdentifier[] = [];
    const identifierMappings = new Map(BUILT_IN_IDENTIFIERS);
    const includeURIs: URI[] = [];

    const addIdentifier = (identifier: LogicScriptIdentifier) => {
      documentIdentifiers.push(identifier);
      if (!identifiersByName.has(identifier.name)) {
        identifiersByName.set(identifier.name, []);
      }
      identifiersByName
        .get(identifier.name)
        ?.push({ fileUri: uri.toString(), identifier });
    };

    const addDefine = (directive: LogicScriptDefineDirective) => {
      const define: LogicScriptDefine = {
        directive: directive,
        fileUri: uri.toString(),
        identifierMapping: buildIdentifierMappingForDefineDirective(
          directive,
          identifierMappings
        ),
      };
      if (identifierMappings.has(directive.identifier.name)) {
        if (directive.identifier.location) {
          diagnostics.push({
            message: `Duplicate definition for ${directive.identifier.name}`,
            range: pegJSLocationRangeToVSCodeRange(
              directive.identifier.location
            ),
          });
        }
      } else {
        identifierMappings.set(
          directive.identifier.name,
          define.identifierMapping
        );
      }
      documentDefines.push(define);

      if (!definesByName.has(directive.identifier.name)) {
        definesByName.set(directive.identifier.name, []);
      }
      definesByName.get(directive.identifier.name)?.push(define);
    };

    const processArgument = (argument: LogicScriptArgument) => {
      if (argument.type === "Identifier") {
        addIdentifier(argument);
      }
    };

    const processBooleanExpression = (
      expression: LogicScriptBooleanExpression
    ) => {
      if (
        expression.type === "AndExpression" ||
        expression.type === "OrExpression"
      ) {
        expression.clauses.forEach(processBooleanExpression);
      } else if (expression.type === "NotExpression") {
        processBooleanExpression(expression.expression);
      } else if (expression.type === "BooleanBinaryOperation") {
        processArgument(expression.left);
        processArgument(expression.right);
      } else if (expression.type === "TestCall") {
        expression.argumentList.forEach(processArgument);
      } else if (expression.type === "Identifier") {
        addIdentifier(expression);
      }
    };

    const processStatement = (statement: LogicScriptStatement) => {
      if (statement.type === "DefineDirective") {
        addDefine(statement);

        if (statement.value.type === "Identifier") {
          addIdentifier(statement.value);
        }
      } else if (statement.type === "IncludeDirective") {
        const includeURI = Utils.resolvePath(
          uri,
          "..",
          statement.filename.value
        );
        documentIncludes.push(statement);
        includeURIs.push(includeURI);
      } else if (statement.type === "CommandCall") {
        statement.argumentList.forEach(processArgument);
      } else if (statement.type === "ArithmeticAssignmentStatement") {
        addIdentifier(statement.assignee);
        if (statement.value.type === "Identifier") {
          addIdentifier(statement.value);
        }
      } else if (statement.type === "IfStatement") {
        processBooleanExpression(statement.conditions);
        statement.thenStatements.forEach(processStatement);
        statement.elseStatements.forEach(processStatement);
      } else if (statement.type === "LeftIndirectAssignmentStatement") {
        addIdentifier(statement.assigneePointer);
        if (statement.value.type === "Identifier") {
          addIdentifier(statement.value);
        }
      } else if (statement.type === "RightIndirectAssignmentStatement") {
        addIdentifier(statement.assignee);
        addIdentifier(statement.valuePointer);
      } else if (statement.type === "UnaryOperationStatement") {
        addIdentifier(statement.identifier);
      } else if (statement.type === "ValueAssignmentStatement") {
        addIdentifier(statement.assignee);
        if (statement.value.type === "Identifier") {
          addIdentifier(statement.value);
        }
      }
    };

    statements.forEach(processStatement);

    definesByDocument.set(uri.toString(), documentDefines);
    includedDocumentsByDocument.set(
      uri.toString(),
      includeURIs.map((includeURI) => includeURI.toString())
    );
    includeDirectivesByDocument.set(uri.toString(), documentIncludes);
    identifiersByDocument.set(uri.toString(), documentIdentifiers);
    await Promise.all(
      includeURIs.map(async (includeURI) => {
        const includedDocument = documents.get(includeURI.toString());
        if (includedDocument) {
          await refreshTextDocument(includeURI, includedDocument.getText());
        } else {
          const includeContents = fs.readFileSync(includeURI.fsPath, "utf-8");
          await refreshTextDocument(includeURI, includeContents);
        }
      })
    );
  }

  // we might have diagnostics even if we don't have statements (e.g. syntax errors)
  connection.sendDiagnostics({ uri: uri.toString(), diagnostics });
}

connection.onDidChangeWatchedFiles((change) => {
  change.changes.forEach((changedFile) => {
    const uri = URI.parse(changedFile.uri);
    if (
      changedFile.type === FileChangeType.Changed ||
      changedFile.type === FileChangeType.Created
    ) {
      refreshTextDocument(uri, fs.readFileSync(uri.fsPath, "utf-8"));
    } else if (changedFile.type === FileChangeType.Deleted) {
      clearDocumentData(uri);
    }
  });
});

function visibleDefinesForDocument(uri: string): LogicScriptDefine[] {
  const visibleDefines = [...(definesByDocument.get(uri) ?? [])];

  (includedDocumentsByDocument.get(uri) ?? []).forEach((includeURI) => {
    visibleDefines.push(...visibleDefinesForDocument(includeURI));
  });

  return visibleDefines;
}

// This handler provides the initial list of the completion items.
connection.onCompletion(
  (textDocumentPosition: TextDocumentPositionParams): CompletionItem[] => {
    const documentDefines = visibleDefinesForDocument(
      textDocumentPosition.textDocument.uri
    );

    return [
      ...agiCommands.map((command) => ({
        label: command.name,
        kind: CompletionItemKind.Function,
        data: command,
      })),
      ...documentDefines.map((define) => ({
        label: define.directive.identifier.name,
        kind: CompletionItemKind.Variable,
        data: define,
      })),
    ];
  }
);

// This handler resolves additional information for the item selected in
// the completion list.
connection.onCompletionResolve(
  (item: CompletionItem): CompletionItem => {
    // if (item.data === 1) {
    //   item.detail = "TypeScript details";
    //   item.documentation = "TypeScript documentation";
    // } else if (item.data === 2) {
    //   item.detail = "JavaScript details";
    //   item.documentation = "JavaScript documentation";
    // }
    return item;
  }
);

function findIdentifierAtLocation(documentUri: string, position: Position) {
  const identifiers = identifiersByDocument.get(documentUri) ?? [];
  const identifier = identifiers.find((identifier) => {
    const { location } = identifier;
    if (!location) {
      return false;
    }

    const range = pegJSLocationRangeToVSCodeRange(location);
    if (range.start.line > position.line || range.end.line < position.line) {
      return false;
    }

    if (
      range.start.character > position.character ||
      range.end.character < position.character
    ) {
      return false;
    }

    return true;
  });

  return identifier;
}

connection.onDefinition((params) => {
  const identifier = findIdentifierAtLocation(
    params.textDocument.uri,
    params.position
  );

  if (!identifier) {
    return;
  }

  const defines = definesByName.get(identifier.name);
  if (!defines) {
    return;
  }

  const definitionLinks: DefinitionLink[] = [];
  defines.forEach((define) => {
    const { location } = define.directive;
    if (!location) {
      return;
    }

    definitionLinks.push({
      targetUri: define.fileUri,
      targetRange: pegJSLocationRangeToVSCodeRange(location),
      targetSelectionRange: pegJSLocationRangeToVSCodeRange(location),
    });
  });
  return definitionLinks;
});

connection.onReferences((params) => {
  const identifier = findIdentifierAtLocation(
    params.textDocument.uri,
    params.position
  );

  if (!identifier) {
    return;
  }

  const identifiers = identifiersByName.get(identifier.name);
  if (!identifiers) {
    return;
  }

  const locations: Location[] = [];
  identifiers.forEach((otherIdentifier) => {
    const { location } = otherIdentifier.identifier;
    if (!location) {
      return;
    }

    locations.push({
      uri: otherIdentifier.fileUri,
      range: pegJSLocationRangeToVSCodeRange(location),
    });
  });
  return locations;
});

connection.onHover((params) => {
  const identifier = findIdentifierAtLocation(
    params.textDocument.uri,
    params.position
  );

  if (!identifier) {
    return;
  }

  const defines = definesByName.get(identifier.name);
  if (!defines) {
    return;
  }

  const firstDefine = defines[0];
  if (!firstDefine) {
    return;
  }

  let description: string;
  if (firstDefine.identifierMapping.identifierType === "constant") {
    description = JSON.stringify(firstDefine.identifierMapping.value);
  } else {
    description = `${firstDefine.identifierMapping.type} #${firstDefine.identifierMapping.number}`;
  }

  return {
    range: identifier.location
      ? pegJSLocationRangeToVSCodeRange(identifier.location)
      : undefined,
    contents: {
      kind: MarkupKind.Markdown,
      value: `\`${identifier.name}\` - ${description}`,
    },
  };
});

connection.onDocumentLinks((params) => {
  const includeDirectives =
    includeDirectivesByDocument.get(params.textDocument.uri) ?? [];

  const links: DocumentLink[] = [];

  includeDirectives.forEach((directive) => {
    const { location } = directive.filename;
    if (!location) {
      return;
    }

    links.push({
      range: pegJSLocationRangeToVSCodeRange(location),
      target: Utils.resolvePath(
        URI.parse(params.textDocument.uri),
        "..",
        directive.filename.value
      ).toString(),
    });
  });

  return links;
});

connection.onDocumentLinkResolve((link) => {
  return link;
});

// Make the text document manager listen on the connection
// for open, change and close text document events
documents.listen(connection);

// Listen on the connection
connection.listen();
