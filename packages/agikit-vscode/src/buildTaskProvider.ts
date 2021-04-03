import * as vscode from 'vscode';
import * as path from 'path';
import * as child_process from 'child_process';
import { ChildProcess } from 'node:child_process';

export function buildTaskProvider(
  context: vscode.ExtensionContext,
): vscode.TaskProvider<vscode.Task> {
  return {
    provideTasks() {
      const workspaceFolders = vscode.workspace.workspaceFolders ?? [];
      const tasks: vscode.Task[] = [];
      workspaceFolders.forEach((workspaceFolder) => {
        const buildTask = new vscode.Task(
          {
            type: 'agikit',
          },
          vscode.TaskScope.Workspace,
          'Build AGI game',
          'agikit',
          new vscode.CustomExecution(async (_resolvedDefinition) => {
            let cp: ChildProcess;
            const writeEmitter = new vscode.EventEmitter<string>();
            const closeEmitter = new vscode.EventEmitter<void | number>();
            const pty: vscode.Pseudoterminal = {
              open: () => {
                const cliPath = context.asAbsolutePath(path.join('dist', 'startCli.js'));
                cp = child_process.fork(cliPath, ['build', '.'], {
                  cwd: workspaceFolder.uri.fsPath,
                  stdio: 'pipe',
                  detached: true,
                });
                cp.on('close', (code) => {
                  closeEmitter.fire(code ?? void 0);
                });
                cp.stdout?.on('data', (chunk) => {
                  writeEmitter.fire(chunk.toString().replace(/(?<!\r)\n/gm, '\r\n'));
                });
                cp.stderr?.on('data', (chunk) => {
                  writeEmitter.fire(chunk.toString().replace(/(?<!\r)\n/gm, '\r\n'));
                });
              },
              close: () => {
                cp.kill();
              },
              onDidWrite: writeEmitter.event,
              onDidClose: closeEmitter.event,
            };
            return pty;
          }),
        );
        buildTask.group = vscode.TaskGroup.Build;
        tasks.push(buildTask);
      });
      return tasks;
    },

    resolveTask(task: vscode.Task) {
      return task;
    },
  };
}
