// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';
import LogicSemanticTokensProvider from './Logic/LogicSemanticTokensProvider';
import { buildTaskProvider } from './buildTaskProvider';
import { runWithScummVM } from './runWithScummVM';
import { PicEditorProvider } from './Pic/PicEditorProvider';
import { ViewEditorProvider } from './View/ViewEditorProvider';
import { SoundEditorProvider } from './Sound/SoundEditorProvider';

let client: LanguageClient;

const AUTODETECT_SCUMMVM_PATHS = [
  '/Applications/ScummVM.app/Contents/MacOS/scummvm',
  'C:\\Program Files\\ScummVM\\ScummVM.exe',
  '/usr/bin/scummvm',
  '/usr/local/bin/scummvm',
];

export function activate(context: vscode.ExtensionContext) {
  const logicSemanticTokensProvider = new LogicSemanticTokensProvider();

  const configuration = vscode.workspace.getConfiguration();
  if (!configuration.get('agikit.scummvmPath')) {
    const autodetectedPath = AUTODETECT_SCUMMVM_PATHS.find((autoPath) => fs.existsSync(autoPath));
    if (autodetectedPath) {
      configuration.update(
        'agikit.scummvmPath',
        autodetectedPath,
        vscode.ConfigurationTarget.Global,
      );
    }
  }

  context.subscriptions.push(
    vscode.languages.registerDocumentSemanticTokensProvider(
      { language: 'agilogic' },
      logicSemanticTokensProvider,
      logicSemanticTokensProvider.legend,
    ),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('agikit.runWithScummVM', () => runWithScummVM(configuration)),
  );

  context.subscriptions.push(PicEditorProvider.register(context));
  context.subscriptions.push(ViewEditorProvider.register(context));
  context.subscriptions.push(SoundEditorProvider.register(context));

  context.subscriptions.push(
    vscode.tasks.registerTaskProvider('agikit', buildTaskProvider(context)),
  );

  let serverModule = context.asAbsolutePath(path.join('dist', 'startServer.js'));
  // The debug options for the server
  // --inspect=6009: runs the server in Node's Inspector mode so VS Code can attach to the server for debugging
  let debugOptions = { execArgv: ['--nolazy', '--inspect=6009'] };

  // If the extension is launched in debug mode then the debug server options are used
  // Otherwise the run options are used
  let serverOptions: ServerOptions = {
    run: {
      module: serverModule,
      transport: TransportKind.ipc,
    },
    debug: {
      module: serverModule,
      transport: TransportKind.ipc,
      options: debugOptions,
    },
  };

  // Options to control the language client
  let clientOptions: LanguageClientOptions = {
    // Register the server for plain text documents
    documentSelector: [{ scheme: 'file', language: 'agilogic' }],
    synchronize: {
      // Notify the server about file changes to '.clientrc files contained in the workspace
      fileEvents: vscode.workspace.createFileSystemWatcher('**/.clientrc'),
    },
  };

  // Create the language client and start the client.
  client = new LanguageClient('agikit', 'agikit language server', serverOptions, clientOptions);

  // Start the client. This will also launch the server
  client.start();
}

export function deactivate() {
  if (!client) {
    return undefined;
  }

  return client.stop();
}
