import { useCallback, useEffect, useMemo, useState } from 'react';
import ReactDOM from 'react-dom';
import { Buffer } from 'buffer';
import { readViewResource } from '@agikit/core/dist/Extract/View/ReadView';
import { buildView } from '@agikit/core/dist/Build/BuildView';
import { ViewEditor } from '../src/ViewEditor';
import { templateEgoBase64 } from './dev-example-data';
import { buildEditingView, buildNonEditingView, EditingView } from '../src/EditingViewTypes';
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
  const [viewResource, setViewResource] = useState<EditingView>(() => {
    const savedView = window.localStorage.getItem('agikit-dev-site:view');
    if (savedView) {
      return buildEditingView(readViewResource(Buffer.from(savedView, 'base64')));
    } else {
      return buildEditingView(templateEgo);
    }
  });
  const [zoom, setZoom] = useState<number>(() => {
    const savedZoom = window.localStorage.getItem('agikit-dev-site:viewZoom');
    if (savedZoom) {
      const parsedZoom = Number.parseInt(savedZoom, 10);
      if (!Number.isNaN(parsedZoom) && parsedZoom >= 1) {
        return parsedZoom;
      }
    }

    return 1;
  });
  const [redoCommands, setRedoCommands] = useState<ViewEditorCommand[]>([]);

  useEffect(() => {
    window.localStorage.setItem(
      'agikit-dev-site:view',
      buildView(
        buildNonEditingView(applyViewEditorCommands(viewResource, viewResource.commands)),
      ).toString('base64'),
    );
  }, [viewResource]);
  useEffect(() => {
    window.localStorage.setItem('agikit-dev-site:viewZoom', zoom.toString());
  }, [zoom]);

  const controlContextValue: ViewEditorControlContextValue = useMemo(
    () => ({
      addCommands: (commands) => {
        setViewResource((prevResource) => {
          return { ...prevResource, commands: [...prevResource.commands, ...commands] };
        });
        setRedoCommands([]);
      },
      confirm: async (message) => window.confirm(message),
      zoom,
      setZoom,
    }),
    [zoom],
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
            let deletedCommands: ViewEditorCommand[] = [];
            setViewResource((prevResource) => {
              deletedCommands = prevResource.commands.filter(
                (cmd) => cmd.uuid === lastCommand.uuid,
              );
              return {
                ...prevResource,
                commands: prevResource.commands.filter((cmd) => cmd.uuid !== lastCommand.uuid),
              };
            });
            setRedoCommands((prevRedoCommands) => [...deletedCommands, ...prevRedoCommands]);
          }
        }
      }

      if (event.key === 's') {
        const builtView = new Blob([
          buildView(
            buildNonEditingView(applyViewEditorCommands(viewResource, viewResource.commands)),
          ),
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
