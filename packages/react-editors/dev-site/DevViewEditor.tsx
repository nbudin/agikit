import { useCallback, useEffect, useMemo, useState } from 'react';
import ReactDOM from 'react-dom';
import { Buffer } from 'buffer';
import { readViewResource } from 'agikit-core/dist/Extract/View/ReadView';
import { buildView } from 'agikit-core/dist/Build/BuildView';
import { ViewEditor } from '../src/ViewEditor';
import { templateEgoBase64 } from './dev-example-data';
import { buildEditingView, EditingView } from '../src/EditingViewTypes';
import {
  ViewEditorControlContext,
  ViewEditorControlContextValue,
} from '../src/ViewEditorControlContext';

import 'bootstrap-icons/font/bootstrap-icons.css';
import './dev-site.css';
import '../styles/common.css';
import '../styles/vieweditor.css';
import { applyViewEditorCommands, ViewEditorCommand } from '../src/ViewEditorCommands';

// @ts-expect-error
window.Buffer = Buffer;

const templateEgo = readViewResource(Buffer.from(templateEgoBase64, 'base64'));

const DevViewEditor = () => {
  const [viewResource, setViewResource] = useState<EditingView>(buildEditingView(templateEgo));
  const [redoCommands, setRedoCommands] = useState<ViewEditorCommand[]>([]);

  const controlContextValue: ViewEditorControlContextValue = useMemo(
    () => ({
      addCommands: (commands) => {
        setViewResource((prevResource) => {
          return { ...prevResource, commands: [...prevResource.commands, ...commands] };
        });
        setRedoCommands([]);
      },
      confirm: async (message) => window.confirm(message),
      deleteCommand: (commandId) => {
        let deletedCommands: ViewEditorCommand[] = [];
        setViewResource((prevResource) => {
          deletedCommands = prevResource.commands.filter((cmd) => cmd.uuid === commandId);
          return {
            ...prevResource,
            commands: prevResource.commands.filter((cmd) => cmd.uuid !== commandId),
          };
        });
        setRedoCommands((prevRedoCommands) => [...deletedCommands, ...prevRedoCommands]);
      },
    }),
    [],
  );

  const keyDownListener = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === 'z' && event.metaKey) {
        if (event.shiftKey) {
          const firstRedoCommand = redoCommands[0];
          if (firstRedoCommand) {
            controlContextValue.addCommands([firstRedoCommand]);
            setRedoCommands((prevRedoCommands) => prevRedoCommands.slice(1));
          }
        } else {
          const lastCommand = viewResource.commands[viewResource.commands.length - 1];
          if (lastCommand) {
            controlContextValue.deleteCommand(lastCommand.uuid);
          }
        }
      }

      if (event.key === 's') {
        const builtView = new Blob([
          buildView(applyViewEditorCommands(viewResource, viewResource.commands)),
        ]);
        const a = document.createElement('a');
        a.href = URL.createObjectURL(builtView);
        a.download = 'saved.agiview';
        a.click();
        URL.revokeObjectURL(a.href);
      }
    },
    [controlContextValue, viewResource, redoCommands],
  );

  useEffect(() => {
    document.addEventListener('keydown', keyDownListener);
    return () => {
      document.removeEventListener('keydown', keyDownListener);
    };
  }, [keyDownListener]);

  return (
    <ViewEditorControlContext.Provider value={controlContextValue}>
      <ViewEditor view={viewResource} />
    </ViewEditorControlContext.Provider>
  );
};

window.addEventListener('load', () => {
  ReactDOM.render(<DevViewEditor />, document.getElementById('view-editor-root'));
});
