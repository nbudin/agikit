import React from 'react';
import { ViewEditorCommand } from './ViewEditorCommands';

export type ViewEditorControlContextValue = {
  confirm: (message: string) => Promise<boolean>;
  addCommands: (commands: ViewEditorCommand[]) => void;
  zoom: number;
  setZoom: React.Dispatch<number>;
};

export const ViewEditorControlContext = React.createContext<ViewEditorControlContextValue>({
  confirm: async () => false,
  addCommands: () => {},
  zoom: 1,
  setZoom: () => {},
});
