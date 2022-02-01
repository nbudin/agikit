import { useEffect, useState } from 'react';
import { Buffer } from 'buffer';
import * as ReactDOM from 'react-dom';

import '../reset.css';
import '../vscode.css';
import 'bootstrap-icons/font/bootstrap-icons.css';
import '@agikit/react-editors/styles/soundeditor.css';
import { IBMPCjrSound } from '@agikit/core';
import SoundEditor from '@agikit/react-editors/dist/SoundEditor';

// @ts-expect-error
window.Buffer = Buffer;

// @ts-expect-error
const vscode = acquireVsCodeApi();
vscode.postMessage({ type: 'ready' });

function VscodeSoundEditor({ initialZoom }: { initialZoom: number }) {
  const [soundResource, setSoundResource] = useState<IBMPCjrSound>({
    toneVoices: [{ notes: [] }, { notes: [] }, { notes: [] }],
    noiseVoice: { notes: [] },
  });
  const [editable, setEditable] = useState(false);
  const [resolveConfirm, setResolveConfirm] = useState<(result: boolean) => void | undefined>();
  // const controlContextValue = useMemo<SoundEditorControlContextValue>(
  //   () => ({
  //     confirm: (message) => {
  //       if (resolveConfirm) {
  //         return Promise.reject('Already showing a confirmation quick pick');
  //       }
  //       return new Promise<boolean>((resolve) => {
  //         setResolveConfirm(() => {
  //           // we have to do it this way because this is the only way to set a function as a state
  //           // value
  //           return resolve;
  //         });
  //         vscode.postMessage({ type: 'confirm', message });
  //       });
  //     },
  //     addCommands: (commands) => {
  //       vscode.postMessage({ type: 'addCommands', commands });
  //     },
  //   }),
  //   [resolveConfirm],
  // );

  useEffect(() => {
    const messageHandler = async (e: MessageEvent) => {
      const { type, body, requestId } = e.data;
      switch (type) {
        case 'init': {
          setEditable(body.editable);
          if (body.untitled) {
            setSoundResource({
              toneVoices: [{ notes: [] }, { notes: [] }, { notes: [] }],
              noiseVoice: { notes: [] },
            });
            return;
          } else {
            setSoundResource(body.resource);
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
          setSoundResource(newResource);
          return;
        }
        case 'getFileData': {
          vscode.postMessage({
            type: 'response',
            requestId,
            // TODO: replace when we can encode sounds
            body: Array.from([]),
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
    // <ViewEditorControlContext.Provider value={controlContextValue}>
    <SoundEditor sound={soundResource} />
    // </ViewEditorControlContext.Provider>
  );
}

const root = document.querySelector('#sound-editor-root');
if (root) {
  const props = JSON.parse(root.getAttribute('data-react-props') ?? '{}');
  ReactDOM.render(<VscodeSoundEditor {...props} />, root);
}
