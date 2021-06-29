import * as vscode from 'vscode';
import * as path from 'path';
import * as child_process from 'child_process';

export async function runWithScummVM(configuration: vscode.WorkspaceConfiguration) {
  const scummvmPath = configuration.get('agikit.scummvmPath');
  if (!scummvmPath) {
    vscode.window.showErrorMessage(
      'ScummVM could not be found.  Please configure the path to ScummVM in your settings.',
    );
  } else {
    const folder = (vscode.workspace.workspaceFolders ?? [])[0];
    if (!folder) {
      vscode.window.showErrorMessage(
        "Can't determine workspace folder.  Please open an AGI game as a Visual Studio Code workspace.",
      );
    } else {
      const tasks = await vscode.tasks.fetchTasks({ type: 'agikit' });
      if (tasks.length === 0) {
        vscode.window.showErrorMessage(
          'No agikit task defined in this workspace.  Please add a build task.',
        );
      } else {
        const execution = await vscode.tasks.executeTask(tasks[0]);
        vscode.tasks.onDidEndTask((e) => {
          if (e.execution === execution) {
            child_process.exec(
              `${scummvmPath} -p "${path.join(folder.uri.fsPath, 'build')}" agi-fanmade`,
              (error) => {
                if (error) {
                  vscode.window.showErrorMessage(error.message);
                }
              },
            );
          }
        });
      }
    }
  }
}
