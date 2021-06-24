import React from 'react';
import { ViewEditorCommand } from './ViewEditorCommands';

export type ViewEditorControlContextValue = {
  confirm: (message: string) => Promise<boolean>;
  addCommands: (commands: ViewEditorCommand[]) => void;
};

export const ViewEditorControlContext = React.createContext<ViewEditorControlContextValue>({
  confirm: async () => false,
  addCommands: () => {},
});
