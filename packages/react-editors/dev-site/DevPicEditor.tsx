import { useMemo, useState } from 'react';
import ReactDOM from 'react-dom';
import { Buffer } from 'buffer';

import { PicEditor } from '../src/PicEditor';
import {
  PicEditorControlContext,
  PicEditorControlContextValue,
} from '../src/PicEditorControlContext';
import {
  EditingPictureCommand,
  EditingPictureResource,
  preparePicCommandForEditing,
} from '../src/EditingPictureTypes';

import 'bootstrap-icons/font/bootstrap-icons.css';
import './dev-site.css';
import '../styles/common.css';
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
              commands: [...commands.map(preparePicCommandForEditing), ...prevResource.commands],
            };
          }

          const afterCommandIndex = prevResource.commands.findIndex(
            (cmd) => cmd.uuid === afterCommandId,
          );
          if (afterCommandIndex > -1) {
            const newCommands = [...prevResource.commands];
            newCommands.splice(
              afterCommandIndex + 1,
              0,
              ...commands.map(preparePicCommandForEditing),
            );
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
      setCommandsEnabled: (enabled: (command: EditingPictureCommand) => boolean) => {
        setPictureResource((prevResource) => ({
          ...prevResource,
          commands: prevResource.commands.map((command) => ({
            ...command,
            enabled: enabled(command),
          })),
        }));
      },
    }),
    [],
  );

  return (
    <PicEditorControlContext.Provider value={controlContextValue}>
      <PicEditor pictureResource={pictureResource} />
    </PicEditorControlContext.Provider>
  );
};

window.addEventListener('load', () => {
  console.log('loaded, rendering!');
  ReactDOM.render(<DevPicEditor />, document.getElementById('pic-editor-root'));
});
