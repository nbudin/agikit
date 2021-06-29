import * as vscode from 'vscode';
import escapeHtml from 'escape-html';
import { readViewResource } from 'agikit-core/dist/Extract/View/ReadView';
import { Disposable, disposeAll } from '../disposable';
import { randomBytes } from 'crypto';
import WebviewCollection from '../WebviewCollection';
import {
  applyViewEditorCommands,
  ViewEditorCommand,
} from '../../../react-editors/dist/ViewEditorCommands';
import { serializeView } from './ViewSerialization';
import { buildEditingView, EditingView } from '../../../react-editors/dist/EditingViewTypes';

interface ViewDocumentDelegate {
  getFileData(): Promise<Uint8Array>;
}

function readDocumentForEditing(content: Buffer): EditingView {
  if (content.byteLength === 0) {
    return {
      description: '',
      loops: [],
      commands: [],
    };
  }

  return buildEditingView(readViewResource(content));
}

export class ViewEditorDocument extends Disposable implements vscode.CustomDocument {
  private readonly _uri: vscode.Uri;
  private readonly _delegate: ViewDocumentDelegate;
  private _resource: EditingView;
  private _edits: Array<ViewEditorCommand> = [];
  private _savedEdits: Array<ViewEditorCommand> = [];

  static async create(
    uri: vscode.Uri,
    backupId: string | undefined,
    delegate: ViewDocumentDelegate,
  ): Promise<ViewEditorDocument> {
    // If we have a backup, read that. Otherwise read the resource from the workspace
    const dataFile = typeof backupId === 'string' ? vscode.Uri.parse(backupId) : uri;
    const data = await this.readFile(dataFile);
    return new ViewEditorDocument(uri, data, delegate);
  }

  private static async readFile(uri: vscode.Uri): Promise<Buffer> {
    if (uri.scheme === 'untitled') {
      return Buffer.alloc(0);
    }
    return Buffer.from(await vscode.workspace.fs.readFile(uri));
  }

  private constructor(uri: vscode.Uri, initialContent: Buffer, delegate: ViewDocumentDelegate) {
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
      readonly content: EditingView;
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

  addCommands(commands: ViewEditorCommand[]) {
    commands.forEach((command) => {
      this._edits.push(command);

      const undo = async () => {
        this._edits.pop();
        this._onDidChangeDocument.fire({
          content: applyViewEditorCommands(this._resource, this._edits),
        });
      };
      const redo = async () => {
        this._edits.push(command);
        this._onDidChangeDocument.fire({
          content: applyViewEditorCommands(this._resource, this._edits),
        });
      };

      this._onDidChange.fire({
        label: command.type,
        undo,
        redo,
      });
    });

    this._onDidChangeDocument.fire({
      content: applyViewEditorCommands(this._resource, this._edits),
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
    const diskContent = await ViewEditorDocument.readFile(this.uri);
    this._resource = readDocumentForEditing(diskContent);
    this._edits = this._savedEdits;
    this._onDidChangeDocument.fire({
      content: applyViewEditorCommands(this._resource, this._edits),
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

export class ViewEditorProvider implements vscode.CustomEditorProvider<ViewEditorDocument> {
  private static newViewFileId = 1;
  private static readonly viewType = 'agikit.viewEditor';
  private readonly webviews = new WebviewCollection();

  public static register(context: vscode.ExtensionContext): vscode.Disposable {
    vscode.commands.registerCommand('agikit.viewEditor.new', () => {
      const workspaceFolders = vscode.workspace.workspaceFolders;
      if (!workspaceFolders) {
        vscode.window.showErrorMessage(
          'Creating new .agiview resources currently requires opening a workspace',
        );
        return;
      }

      const uri = vscode.Uri.joinPath(
        workspaceFolders[0].uri,
        `new-${ViewEditorProvider.newViewFileId++}.agiview`,
      ).with({ scheme: 'untitled' });

      vscode.commands.executeCommand('vscode.openWith', uri, ViewEditorProvider.viewType);
    });

    return vscode.window.registerCustomEditorProvider(
      ViewEditorProvider.viewType,
      new ViewEditorProvider(context),
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
    vscode.CustomDocumentEditEvent<ViewEditorDocument>
  >();
  public readonly onDidChangeCustomDocument = this._onDidChangeCustomDocument.event;

  constructor(private readonly _context: vscode.ExtensionContext) {}

  public saveCustomDocument(
    document: ViewEditorDocument,
    cancellation: vscode.CancellationToken,
  ): Thenable<void> {
    return document.save(cancellation);
  }

  public saveCustomDocumentAs(
    document: ViewEditorDocument,
    destination: vscode.Uri,
    cancellation: vscode.CancellationToken,
  ): Thenable<void> {
    return document.saveAs(destination, cancellation);
  }

  revertCustomDocument(
    document: ViewEditorDocument,
    cancellation: vscode.CancellationToken,
  ): Thenable<void> {
    return document.revert(cancellation);
  }

  backupCustomDocument(
    document: ViewEditorDocument,
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
    const document: ViewEditorDocument = await ViewEditorDocument.create(
      uri,
      openContext.backupId,
      {
        getFileData: async () => {
          const webviewsForDocument = Array.from(this.webviews.get(document.uri));
          if (!webviewsForDocument.length) {
            throw new Error('Could not find webview to save for');
          }
          const panel = webviewsForDocument[0];
          const response = await this.postMessageWithResponse<number[]>(panel, 'getFileData', {});
          return new Uint8Array(response);
        },
      },
    );

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
            content: serializeView(e.content),
          });
        }
      }),
    );

    document.onDidDispose(() => disposeAll(listeners));

    return document;
  }

  async resolveCustomEditor(
    document: ViewEditorDocument,
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
            resource: serializeView(document.resource),
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
      } else if (e.type === 'addCommands') {
        document.addCommands(e.commands);
      } else if (e.type === 'setZoom') {
        const configuration = vscode.workspace.getConfiguration();
        configuration.update('agikit.viewEditor.zoom', e.zoom);
      }
    });
  }

  private getHtmlForWebview(webview: vscode.Webview): string {
    // Local path to script and css for the webview
    const scriptUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this._context.extensionUri, 'dist', 'VscodeViewEditor.js'),
    );

    const styleMainUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this._context.extensionUri, 'dist', 'VscodeViewEditor.css'),
    );

    // Use a nonce to whitelist which scripts can be run
    const nonce = randomBytes(16).toString('base64');

    const configuration = vscode.workspace.getConfiguration();
    const initialZoom = configuration.get('agikit.viewEditor.zoom', 6);

    return /* html */ `
			<!DOCTYPE html>
			<html lang="en">
			<head>
				<meta charset="UTF-8">
				<meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src ${
          webview.cspSource
        } blob:; style-src ${webview.cspSource}; font-src ${
      webview.cspSource
    }; script-src 'nonce-${nonce}';">
				<meta name="viewport" content="width=device-width, initial-scale=1.0">
				<link href="${styleMainUri}" rel="stylesheet" />
				<title>AGI VIEW Editor</title>
			</head>
			<body>
				<div id="view-editor-root" data-react-props="${escapeHtml(JSON.stringify({ initialZoom }))}"></div>

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

  private onMessage(document: ViewEditorDocument, message: any) {
    switch (message.type) {
      case 'response': {
        const callback = this._callbacks.get(message.requestId);
        callback?.(message.body);
        return;
      }
    }
  }
}
