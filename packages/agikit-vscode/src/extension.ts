// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from "vscode";
import * as path from "path";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";
import LogicSemanticTokensProvider from "./Logic/logicSemanticTokensProvider";

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
  const logicSemanticTokensProvider = new LogicSemanticTokensProvider();

  context.subscriptions.push(
    vscode.languages.registerDocumentSemanticTokensProvider(
      { language: "agilogic" },
      logicSemanticTokensProvider,
      logicSemanticTokensProvider.legend
    )
  );

  let serverModule = context.asAbsolutePath(
    path.join("node_modules", "agikit-logic-language-server", "dist", "main.js")
  );
  // The debug options for the server
  // --inspect=6009: runs the server in Node's Inspector mode so VS Code can attach to the server for debugging
  let debugOptions = { execArgv: ["--nolazy", "--inspect=6009"] };

  // If the extension is launched in debug mode then the debug server options are used
  // Otherwise the run options are used
  let serverOptions: ServerOptions = {
    run: { module: serverModule, transport: TransportKind.ipc },
    debug: {
      module: serverModule,
      transport: TransportKind.ipc,
      options: debugOptions,
    },
  };

  // Options to control the language client
  let clientOptions: LanguageClientOptions = {
    // Register the server for plain text documents
    documentSelector: [{ scheme: "file", language: "agilogic" }],
    synchronize: {
      // Notify the server about file changes to '.clientrc files contained in the workspace
      fileEvents: vscode.workspace.createFileSystemWatcher("**/.clientrc"),
    },
  };

  // Create the language client and start the client.
  client = new LanguageClient(
    "agikit",
    "agikit language server",
    serverOptions,
    clientOptions
  );

  // Start the client. This will also launch the server
  client.start();
}

export function deactivate() {
  if (!client) {
    return undefined;
  }

  return client.stop();
}
