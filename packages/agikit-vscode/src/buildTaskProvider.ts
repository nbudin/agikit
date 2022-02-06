import * as vscode from 'vscode';
import { Logger, Project, ProjectBuilder } from '@agikit/core';
import { bgGreenBright, bgRedBright, bgYellowBright, black } from 'ansi-colors';

class PseudoterminalLogger extends Logger {
  pty: vscode.Pseudoterminal;
  openEmitter: vscode.EventEmitter<void>;
  writeEmitter: vscode.EventEmitter<string>;
  closeEmitter: vscode.EventEmitter<number>;

  constructor() {
    super();
    this.openEmitter = new vscode.EventEmitter<void>();
    this.writeEmitter = new vscode.EventEmitter<string>();
    this.closeEmitter = new vscode.EventEmitter<number>();
    this.pty = {
      open: () => {
        this.openEmitter.fire();
      },
      close: () => {},
      onDidWrite: this.writeEmitter.event,
      onDidClose: this.closeEmitter.event,
    };
  }

  error(message: string): void {
    this.writeEmitter.fire(`${bgRedBright(black('[ERROR]'))} ${message}\r\n`);
  }

  warn(message: string): void {
    this.writeEmitter.fire(`${bgYellowBright(black('[WARN]'))}  ${message}\r\n`);
  }

  log(message: string): void {
    this.writeEmitter.fire(`${bgGreenBright(black('[INFO]'))}  ${message}\r\n`);
  }
}

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
            type: 'agikit-build',
          },
          vscode.TaskScope.Workspace,
          'Build AGI game',
          'agikit',
          new vscode.CustomExecution(async (_resolvedDefinition) => {
            const logger = new PseudoterminalLogger();
            const builder = new ProjectBuilder(new Project(workspaceFolder.uri.fsPath), logger);

            logger.openEmitter.event(() => {
              logger.log(`Starting builder for ${builder.project.basePath}`);
              try {
                builder.buildProject();
                logger.log(`Build complete`);
                logger.closeEmitter.fire(0);
              } catch (error) {
                if (error instanceof Error) {
                  logger.error(error.message);
                } else {
                  logger.error(`Build failed: ${error}`);
                }
                logger.closeEmitter.fire(1);
              }
            });

            return logger.pty;
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
