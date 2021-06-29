import { useEffect, useMemo, useState } from 'react';
import { buildView } from '@agikit/core/dist/Build/BuildView';
import { Buffer } from 'buffer';
import * as ReactDOM from 'react-dom';
import { ViewEditor } from '@agikit/react-editors/dist/ViewEditor';
import {
  buildEditingView,
  buildNonEditingView,
  EditingView,
} from '@agikit/react-editors/dist/EditingViewTypes';
import {
  ViewEditorControlContext,
  ViewEditorControlContextValue,
} from '@agikit/react-editors/dist/ViewEditorControlContext';

import '../reset.css';
import '../vscode.css';
import 'bootstrap-icons/font/bootstrap-icons.css';
import '@agikit/react-editors/styles/vieweditor.css';
import { deserializeView } from './ViewSerialization';

// @ts-expect-error
window.Buffer = Buffer;

// @ts-expect-error
const vscode = acquireVsCodeApi();
vscode.postMessage({ type: 'ready' });

function VscodePicEditor({ initialZoom }: { initialZoom: number }) {
  const [viewResource, setViewResource] = useState<EditingView>({
    description: undefined,
    loops: [],
    commands: [],
  });
  const [zoom, setZoom] = useState(initialZoom ?? 6);
  const [editable, setEditable] = useState(false);
  const [resolveConfirm, setResolveConfirm] = useState<(result: boolean) => void | undefined>();
  const controlContextValue = useMemo<ViewEditorControlContextValue>(
    () => ({
      confirm: (message) => {
        if (resolveConfirm) {
          return Promise.reject('Already showing a confirmation quick pick');
        }
        return new Promise<boolean>((resolve) => {
          setResolveConfirm(() => {
            // we have to do it this way because this is the only way to set a function as a state
            // value
            return resolve;
          });
          vscode.postMessage({ type: 'confirm', message });
        });
      },
      addCommands: (commands) => {
        vscode.postMessage({ type: 'addCommands', commands });
      },
      zoom,
      setZoom: (zoom) => {
        setZoom(zoom);
        vscode.postMessage({ type: 'setZoom', zoom });
      },
    }),
    [resolveConfirm, zoom],
  );

  useEffect(() => {
    const messageHandler = async (e: MessageEvent) => {
      const { type, body, requestId } = e.data;
      switch (type) {
        case 'init': {
          setEditable(body.editable);
          if (body.untitled) {
            setViewResource({ loops: [], description: undefined, commands: [] });
            return;
          } else {
            setViewResource(deserializeView(body.resource));
            return;
          }
        }
        case 'confirmResult':
          const result = body.confirmed;
          if (resolveConfirm) {
            resolveConfirm(result);
          }
          setResolveConfirm(undefined);
          return;
        case 'update': {
          const newResource = body.content;
          setViewResource(deserializeView(newResource));
          return;
        }
        case 'getFileData': {
          vscode.postMessage({
            type: 'response',
            requestId,
            body: Array.from(buildView(buildNonEditingView(viewResource))),
          });
          return;
        }
      }
    };
    window.addEventListener('message', messageHandler);

    return () => {
      window.removeEventListener('message', messageHandler);
    };
  });

  return (
    <ViewEditorControlContext.Provider value={controlContextValue}>
      <ViewEditor view={viewResource} />
    </ViewEditorControlContext.Provider>
  );
}

const root = document.querySelector('#view-editor-root');
if (root) {
  const props = JSON.parse(root.getAttribute('data-react-props') ?? '{}');
  ReactDOM.render(<VscodePicEditor {...props} />, root);
}
