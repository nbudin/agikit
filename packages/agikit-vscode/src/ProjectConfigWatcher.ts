import * as vscode from 'vscode';
import { isPresent } from 'ts-is-present';
import { DefaultProjectConfig, ProjectConfig, readProjectConfig } from '@agikit/core';

export class ProjectConfigWatcher {
  private watcher: vscode.FileSystemWatcher;
  currentUri: vscode.Uri | undefined;
  projectConfig: ProjectConfig;
  onChange: vscode.EventEmitter<{ projectConfig: ProjectConfig }>;

  constructor() {
    this.projectConfig = DefaultProjectConfig;
    this.onChange = new vscode.EventEmitter<{ projectConfig: ProjectConfig }>();

    this.watcher = vscode.workspace.createFileSystemWatcher('**/agikit-project.json');
    this.watcher.onDidChange((uri) => {
      if (uri.toString() === this.currentUri?.toString()) {
        console.log('reloading project config');
        this.readProjectConfig(uri);
      }
    });
    this.watcher.onDidCreate(() => {
      this.findProjectConfigInWorkspace();
    });
    this.watcher.onDidDelete(() => {
      this.findProjectConfigInWorkspace();
    });

    this.findProjectConfigInWorkspace();
  }

  async findProjectConfigInWorkspace() {
    const possibleProjectConfigUris =
      vscode.workspace.workspaceFolders?.map((folder) =>
        vscode.Uri.joinPath(folder.uri, 'agikit-project.json'),
      ) ?? [];

    const firstFoundProjectConfigUri = (
      await Promise.all(
        possibleProjectConfigUris.map((uri) =>
          vscode.workspace.fs.stat(uri).then(
            () => uri,
            () => undefined,
          ),
        ),
      )
    ).find(isPresent);

    this.currentUri = firstFoundProjectConfigUri;
    if (this.currentUri) {
      console.log(`loading project config from ${this.currentUri}`);
      this.readProjectConfig(this.currentUri);
    } else {
      console.log('loading default project config because no file was found');
      this.projectConfig = DefaultProjectConfig;
      this.onChange.fire({ projectConfig: this.projectConfig });
    }
  }

  async readProjectConfig(uri: vscode.Uri) {
    const data = Buffer.from(await vscode.workspace.fs.readFile(uri));
    const newConfig = readProjectConfig(data);
    this.projectConfig = newConfig;
    this.onChange.fire({ projectConfig: newConfig });
  }
}
