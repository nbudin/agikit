import React from 'react';
import { EditingPictureCommand } from './EditingPictureTypes';

export type PicEditorControlContextValue = {
  confirm: (message: string) => Promise<boolean>;
  addCommands: (commands: EditingPictureCommand[], afterCommandId: string | undefined) => void;
  deleteCommand: (commandId: string | undefined) => void;
  setCommandsEnabled: (enabled: (command: EditingPictureCommand) => boolean) => void;
};

export const PicEditorControlContext = React.createContext<PicEditorControlContextValue>({
  confirm: async () => false,
  addCommands: () => {},
  deleteCommand: () => {},
  setCommandsEnabled: () => {},
});
