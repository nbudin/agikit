import { PictureCommand } from 'agikit-core/dist/Types/Picture';
import React from 'react';

export type PicEditorControlContextValue = {
  confirm: (message: string) => Promise<boolean>;
  addCommands: (commands: PictureCommand[], afterCommandId: string | undefined) => void;
  deleteCommand: (commandId: string | undefined) => void;
};

export const PicEditorControlContext = React.createContext<PicEditorControlContextValue>({
  confirm: async () => false,
  addCommands: () => {},
  deleteCommand: () => {},
});
