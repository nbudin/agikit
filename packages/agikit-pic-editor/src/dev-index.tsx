import { useMemo, useState } from 'react';
import ReactDOM from 'react-dom';
import { Buffer } from 'buffer';

import { PicEditor } from './PicEditor';
import { PicEditorControlContext, PicEditorControlContextValue } from './PicEditorControlContext';
import { EditingPictureResource, prepareCommandForEditing } from './EditingPictureTypes';

import 'bootstrap-icons/font/bootstrap-icons.css';
import '../styles/dev-editor.css';
import '../styles/piceditor.css';

// @ts-expect-error
window.Buffer = Buffer;

const DevPicEditor = () => {
  const [pictureResource, setPictureResource] = useState<EditingPictureResource>({
    commands: [],
  });

  const controlContextValue: PicEditorControlContextValue = useMemo(
    () => ({
      addCommands: (commands, afterCommandId) => {
        setPictureResource((prevResource) => {
          if (afterCommandId == null) {
            return {
              ...prevResource,
              commands: [...commands.map(prepareCommandForEditing), ...prevResource.commands],
            };
          }

          const afterCommandIndex = prevResource.commands.findIndex(
            (cmd) => cmd.uuid === afterCommandId,
          );
          if (afterCommandIndex > -1) {
            const newCommands = [...prevResource.commands];
            newCommands.splice(afterCommandIndex + 1, 0, ...commands.map(prepareCommandForEditing));
            return { ...prevResource, commands: newCommands };
          }

          return prevResource;
        });
      },
      confirm: async (message) => window.confirm(message),
      deleteCommand: (commandId) => {
        setPictureResource((prevResource) => ({
          ...prevResource,
          commands: prevResource.commands.filter((cmd) => cmd.uuid !== commandId),
        }));
      },
    }),
    [],
  );

  return (
    <PicEditorControlContext.Provider value={controlContextValue}>
      <PicEditor pictureResource={pictureResource} setPictureResource={setPictureResource} />
    </PicEditorControlContext.Provider>
  );
};

window.addEventListener('load', () => {
  console.log('loaded, rendering!');
  ReactDOM.render(<DevPicEditor />, document.getElementById('pic-editor-root'));
});
