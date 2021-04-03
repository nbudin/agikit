import * as vscode from 'vscode';
import { readPictureResource } from 'agikit-core/dist/Extract/Picture/ReadPicture';
import { Disposable, disposeAll } from '../disposable';
import { randomBytes } from 'crypto';
import {
  applyEditsToResource,
  EditingPictureResource,
  PicDocumentEdit,
  prepareCommandForEditing,
} from './EditingPictureTypes';

interface PicDocumentDelegate {
  getFileData(): Promise<Uint8Array>;
}

class WebviewCollection {
  private readonly _webviews = new Set<{
    readonly resource: string;
    readonly webviewPanel: vscode.WebviewPanel;
  }>();

  /**
   * Get all known webviews for a given uri.
   */
  public *get(uri: vscode.Uri): Iterable<vscode.WebviewPanel> {
    const key = uri.toString();
    for (const entry of this._webviews) {
      if (entry.resource === key) {
        yield entry.webviewPanel;
      }
    }
  }

  /**
   * Add a new webview to the collection.
   */
  public add(uri: vscode.Uri, webviewPanel: vscode.WebviewPanel) {
    const entry = { resource: uri.toString(), webviewPanel };
    this._webviews.add(entry);

    webviewPanel.onDidDispose(() => {
      this._webviews.delete(entry);
    });
  }
}

function readDocumentForEditing(content: Buffer): EditingPictureResource {
  const resource = readPictureResource(content);
  return {
    ...resource,
    commands: resource.commands.map(prepareCommandForEditing),
  };
}

export class PicEditorDocument extends Disposable implements vscode.CustomDocument {
  private readonly _uri: vscode.Uri;
  private readonly _delegate: PicDocumentDelegate;
  private _resource: EditingPictureResource;
  private _edits: Array<PicDocumentEdit> = [];
  private _savedEdits: Array<PicDocumentEdit> = [];

  static async create(
    uri: vscode.Uri,
    backupId: string | undefined,
    delegate: PicDocumentDelegate,
  ): Promise<PicEditorDocument> {
    // If we have a backup, read that. Otherwise read the resource from the workspace
    const dataFile = typeof backupId === 'string' ? vscode.Uri.parse(backupId) : uri;
    const data = await this.readFile(dataFile);
    return new PicEditorDocument(uri, data, delegate);
  }

  private static async readFile(uri: vscode.Uri): Promise<Buffer> {
    if (uri.scheme === 'untitled') {
      return Buffer.alloc(0);
    }
    return Buffer.from(await vscode.workspace.fs.readFile(uri));
  }

  private constructor(uri: vscode.Uri, initialContent: Buffer, delegate: PicDocumentDelegate) {
    super();

    this._uri = uri;
    this._resource = readDocumentForEditing(initialContent);
    this._delegate = delegate;
  }

  public get uri() {
    return this._uri;
  }

  public get resource() {
    return this._resource;
  }

  private readonly _onDidDispose = this._register(new vscode.EventEmitter<void>());

  // Fired when the document is disposed of.
  public readonly onDidDispose = this._onDidDispose.event;

  private readonly _onDidChangeDocument = this._register(
    new vscode.EventEmitter<{
      readonly content: EditingPictureResource;
    }>(),
  );

  // Fired to notify webviews that the document has changed.
  public readonly onDidChangeContent = this._onDidChangeDocument.event;

  private readonly _onDidChange = this._register(
    new vscode.EventEmitter<{
      readonly label: string;
      undo(): void;
      redo(): void;
    }>(),
  );

  public readonly onDidChange = this._onDidChange.event;

  dispose(): void {
    this._onDidDispose.fire();
    super.dispose();
  }

  makeEdit(edit: PicDocumentEdit) {
    this._edits.push(edit);

    const undo = async () => {
      this._edits.pop();
      this._onDidChangeDocument.fire({
        content: applyEditsToResource(this._resource, this._edits),
      });
    };
    const redo = async () => {
      this._edits.push(edit);
      this._onDidChangeDocument.fire({
        content: applyEditsToResource(this._resource, this._edits),
      });
    };

    if (edit.type === 'AddCommands') {
      this._onDidChange.fire({
        label: edit.commands.map((c) => c.type).join(', '),
        undo,
        redo,
      });
    } else {
      const command = this._resource.commands.find((c) => c.uuid === edit.commandId);
      this._onDidChange.fire({
        label: `Remove ${command?.type ?? 'command'}`,
        undo,
        redo,
      });
    }

    this._onDidChangeDocument.fire({
      content: applyEditsToResource(this._resource, this._edits),
    });
  }

  async save(cancellation: vscode.CancellationToken): Promise<void> {
    await this.saveAs(this.uri, cancellation);
    this._savedEdits = Array.from(this._edits);
  }

  async saveAs(targetResource: vscode.Uri, cancellation: vscode.CancellationToken): Promise<void> {
    const fileData = await this._delegate.getFileData();
    if (cancellation.isCancellationRequested) {
      return;
    }
    await vscode.workspace.fs.writeFile(targetResource, fileData);
  }

  async revert(_cancellation: vscode.CancellationToken): Promise<void> {
    const diskContent = await PicEditorDocument.readFile(this.uri);
    this._resource = readDocumentForEditing(diskContent);
    this._edits = this._savedEdits;
    this._onDidChangeDocument.fire({
      content: applyEditsToResource(this._resource, this._edits),
    });
  }

  async backup(
    destination: vscode.Uri,
    cancellation: vscode.CancellationToken,
  ): Promise<vscode.CustomDocumentBackup> {
    await this.saveAs(destination, cancellation);

    return {
      id: destination.toString(),
      delete: async () => {
        try {
          await vscode.workspace.fs.delete(destination);
        } catch {
          // noop
        }
      },
    };
  }
}

export class PicEditorProvider implements vscode.CustomEditorProvider<PicEditorDocument> {
  private static newPicFileId = 1;
  private static readonly viewType = 'agikit.picEditor';
  private readonly webviews = new WebviewCollection();

  public static register(context: vscode.ExtensionContext): vscode.Disposable {
    vscode.commands.registerCommand('agikit.picEditor.new', () => {
      const workspaceFolders = vscode.workspace.workspaceFolders;
      if (!workspaceFolders) {
        vscode.window.showErrorMessage(
          'Creating new .agipic resources currently requires opening a workspace',
        );
        return;
      }

      const uri = vscode.Uri.joinPath(
        workspaceFolders[0].uri,
        `new-${PicEditorProvider.newPicFileId++}.agipic`,
      ).with({ scheme: 'untitled' });

      vscode.commands.executeCommand('vscode.openWith', uri, PicEditorProvider.viewType);
    });

    return vscode.window.registerCustomEditorProvider(
      PicEditorProvider.viewType,
      new PicEditorProvider(context),
      {
        // For this demo extension, we enable `retainContextWhenHidden` which keeps the
        // webview alive even when it is not visible. You should avoid using this setting
        // unless is absolutely required as it does have memory overhead.
        webviewOptions: {
          retainContextWhenHidden: true,
        },
        supportsMultipleEditorsPerDocument: false,
      },
    );
  }

  private readonly _onDidChangeCustomDocument = new vscode.EventEmitter<
    vscode.CustomDocumentEditEvent<PicEditorDocument>
  >();
  public readonly onDidChangeCustomDocument = this._onDidChangeCustomDocument.event;

  constructor(private readonly _context: vscode.ExtensionContext) {}

  public saveCustomDocument(
    document: PicEditorDocument,
    cancellation: vscode.CancellationToken,
  ): Thenable<void> {
    return document.save(cancellation);
  }

  public saveCustomDocumentAs(
    document: PicEditorDocument,
    destination: vscode.Uri,
    cancellation: vscode.CancellationToken,
  ): Thenable<void> {
    return document.saveAs(destination, cancellation);
  }

  revertCustomDocument(
    document: PicEditorDocument,
    cancellation: vscode.CancellationToken,
  ): Thenable<void> {
    throw new Error('Method not implemented.');
  }

  backupCustomDocument(
    document: PicEditorDocument,
    context: vscode.CustomDocumentBackupContext,
    cancellation: vscode.CancellationToken,
  ): Thenable<vscode.CustomDocumentBackup> {
    return document.backup(context.destination, cancellation);
  }

  async openCustomDocument(
    uri: vscode.Uri,
    openContext: vscode.CustomDocumentOpenContext,
    token: vscode.CancellationToken,
  ) {
    const document: PicEditorDocument = await PicEditorDocument.create(uri, openContext.backupId, {
      getFileData: async () => {
        const webviewsForDocument = Array.from(this.webviews.get(document.uri));
        if (!webviewsForDocument.length) {
          throw new Error('Could not find webview to save for');
        }
        const panel = webviewsForDocument[0];
        const response = await this.postMessageWithResponse<number[]>(panel, 'getFileData', {});
        return new Uint8Array(response);
      },
    });

    const listeners: vscode.Disposable[] = [];

    listeners.push(
      document.onDidChange((e) => {
        // Tell VS Code that the document has been edited by the use.
        this._onDidChangeCustomDocument.fire({
          document,
          ...e,
        });
      }),
    );

    listeners.push(
      document.onDidChangeContent((e) => {
        // Update all webviews when the document changes
        for (const webviewPanel of this.webviews.get(document.uri)) {
          this.postMessage(webviewPanel, 'update', {
            // edits: e.edits,
            content: e.content,
          });
        }
      }),
    );

    document.onDidDispose(() => disposeAll(listeners));

    return document;
  }

  async resolveCustomEditor(
    document: PicEditorDocument,
    webviewPanel: vscode.WebviewPanel,
    token: vscode.CancellationToken,
  ): Promise<void> {
    // Add the webview to our internal set of active webviews
    this.webviews.add(document.uri, webviewPanel);

    // Setup initial content for the webview
    webviewPanel.webview.options = {
      enableScripts: true,
    };
    webviewPanel.webview.html = this.getHtmlForWebview(webviewPanel.webview);

    webviewPanel.webview.onDidReceiveMessage((e) => this.onMessage(document, e));

    // Wait for the webview to be properly ready before we init
    webviewPanel.webview.onDidReceiveMessage((e) => {
      if (e.type === 'ready') {
        if (document.uri.scheme === 'untitled') {
          this.postMessage(webviewPanel, 'init', {
            untitled: true,
            editable: true,
          });
        } else {
          const editable = vscode.workspace.fs.isWritableFileSystem(document.uri.scheme);

          this.postMessage(webviewPanel, 'init', {
            resource: document.resource,
            editable,
          });
        }
      } else if (e.type === 'confirm') {
        vscode.window.showInformationMessage(e.message, { modal: true }, 'OK').then((button) => {
          webviewPanel.webview.postMessage({
            type: 'confirmResult',
            body: { confirmed: button === 'OK' },
          });
        });
      } else if (e.type === 'deleteCommand') {
        document.makeEdit({
          type: 'DeleteCommand',
          commandId: e.commandId,
        });
      } else if (e.type === 'addCommands') {
        document.makeEdit({
          type: 'AddCommands',
          afterCommandId: e.afterCommandId,
          commands: e.commands,
        });
      }
    });
  }

  private getHtmlForWebview(webview: vscode.Webview): string {
    // Local path to script and css for the webview
    const scriptUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this._context.extensionUri, 'dist', 'VscodePicEditor.js'),
    );

    const styleMainUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this._context.extensionUri, 'dist', 'VscodePicEditor.css'),
    );

    // Use a nonce to whitelist which scripts can be run
    const nonce = randomBytes(16).toString('base64');

    return /* html */ `
			<!DOCTYPE html>
			<html lang="en">
			<head>
				<meta charset="UTF-8">
				<meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src ${webview.cspSource} blob:; style-src ${webview.cspSource}; font-src ${webview.cspSource}; script-src 'nonce-${nonce}';">
				<meta name="viewport" content="width=device-width, initial-scale=1.0">
				<link href="${styleMainUri}" rel="stylesheet" />
				<title>AGI PIC Editor</title>
			</head>
			<body>
				<div id="pic-editor-root"></div>

				<script nonce="${nonce}" src="${scriptUri}"></script>
			</body>
			</html>`;
  }

  private _requestId = 1;
  private readonly _callbacks = new Map<number, (response: any) => void>();

  private postMessageWithResponse<R = unknown>(
    panel: vscode.WebviewPanel,
    type: string,
    body: any,
  ): Promise<R> {
    const requestId = this._requestId++;
    const p = new Promise<R>((resolve) => this._callbacks.set(requestId, resolve));
    panel.webview.postMessage({ type, requestId, body });
    return p;
  }

  private postMessage(panel: vscode.WebviewPanel, type: string, body: any): void {
    panel.webview.postMessage({ type, body });
  }

  private onMessage(document: PicEditorDocument, message: any) {
    switch (message.type) {
      case 'response': {
        const callback = this._callbacks.get(message.requestId);
        callback?.(message.body);
        return;
      }
    }
  }
}
