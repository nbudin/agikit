// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from "vscode";
import * as path from "path";
import * as fs from "fs";
import * as child_process from "child_process";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";
import LogicSemanticTokensProvider from "./Logic/logicSemanticTokensProvider";
import { ChildProcess } from "node:child_process";

let client: LanguageClient;

const AUTODETECT_SCUMMVM_PATHS = [
  "/Applications/ScummVM.app/Contents/MacOS/scummvm",
  "C:\\Program Files\\ScummVM\\ScummVM.exe",
  "/usr/bin/scummvm",
  "/usr/local/bin/scummvm",
];

export function activate(context: vscode.ExtensionContext) {
  const logicSemanticTokensProvider = new LogicSemanticTokensProvider();

  const configuration = vscode.workspace.getConfiguration();
  if (!configuration.get("agikit.scummvmPath")) {
    const autodetectedPath = AUTODETECT_SCUMMVM_PATHS.find((autoPath) =>
      fs.existsSync(autoPath)
    );
    if (autodetectedPath) {
      configuration.update(
        "agikit.scummvmPath",
        autodetectedPath,
        vscode.ConfigurationTarget.Global
      );
    }
  }

  context.subscriptions.push(
    vscode.languages.registerDocumentSemanticTokensProvider(
      { language: "agilogic" },
      logicSemanticTokensProvider,
      logicSemanticTokensProvider.legend
    )
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("agikit.runWithScummVM", async () => {
      const scummvmPath = configuration.get("agikit.scummvmPath");
      if (!scummvmPath) {
        vscode.window.showErrorMessage(
          "ScummVM could not be found.  Please configure the path to ScummVM in your settings."
        );
      } else {
        const folder = (vscode.workspace.workspaceFolders ?? [])[0];
        if (!folder) {
          vscode.window.showErrorMessage(
            "Can't determine workspace folder.  Please open an AGI game as a Visual Studio Code workspace."
          );
        } else {
          const tasks = await vscode.tasks.fetchTasks({ type: "agikit" });
          if (tasks.length === 0) {
            vscode.window.showErrorMessage(
              "No agikit task defined in this workspace.  Please add a build task."
            );
          } else {
            await vscode.tasks.executeTask(tasks[0]);
            child_process.exec(
              `${scummvmPath} -p "${path.join(
                folder.uri.fsPath,
                "build"
              )}" agi-fanmade`,
              (error) => {
                if (error) {
                  vscode.window.showErrorMessage(error.message);
                }
              }
            );
          }
        }
      }
    })
  );

  context.subscriptions.push(
    vscode.tasks.registerTaskProvider("agikit", {
      provideTasks() {
        const workspaceFolders = vscode.workspace.workspaceFolders ?? [];
        const tasks: vscode.Task[] = [];
        workspaceFolders.forEach((workspaceFolder) => {
          const buildTask = new vscode.Task(
            {
              type: "agikit",
            },
            vscode.TaskScope.Workspace,
            "Build AGI game",
            "agikit",
            new vscode.CustomExecution(async (_resolvedDefinition) => {
              let cp: ChildProcess;
              const writeEmitter = new vscode.EventEmitter<string>();
              const closeEmitter = new vscode.EventEmitter<void | number>();
              const pty: vscode.Pseudoterminal = {
                open: () => {
                  const cliPath = context.asAbsolutePath(
                    path.join("dist", "startCli.js")
                  );
                  cp = child_process.fork(
                    cliPath,
                    ["build", "src/", "build/"],
                    {
                      cwd: workspaceFolder.uri.fsPath,
                      stdio: "pipe",
                      detached: true,
                    }
                  );
                  cp.on("close", (code) => {
                    closeEmitter.fire(code ?? void 0);
                  });
                  cp.stdout?.on("data", (chunk) => {
                    writeEmitter.fire(
                      chunk.toString().replace(/(?<!\r)\n/gm, "\r\n")
                    );
                  });
                  cp.stderr?.on("data", (chunk) => {
                    writeEmitter.fire(
                      chunk.toString().replace(/(?<!\r)\n/gm, "\r\n")
                    );
                  });
                },
                close: () => {
                  cp.kill();
                },
                onDidWrite: writeEmitter.event,
                onDidClose: closeEmitter.event,
              };
              return pty;
            })
          );
          buildTask.group = vscode.TaskGroup.Build;
          tasks.push(buildTask);
        });
        return tasks;
      },

      resolveTask(task: vscode.Task) {
        return task;
      },
    })
  );

  let serverModule = context.asAbsolutePath(
    path.join("dist", "startServer.js")
  );
  // The debug options for the server
  // --inspect=6009: runs the server in Node's Inspector mode so VS Code can attach to the server for debugging
  let debugOptions = { execArgv: ["--nolazy", "--inspect=6009"] };

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
