import React from 'react';
import { AGIView } from '../../agikit-core/dist/Types/View';
import { EditingView } from './EditingViewTypes';
import { ViewEditorCommand } from './ViewEditorCommands';

export type ViewEditorContextValue = {
  view: EditingView;
  viewWithCommandsApplied: AGIView;
  loopNumber: number;
  setLoopNumber: React.Dispatch<React.SetStateAction<number>>;
  celNumber: number;
  setCelNumber: React.Dispatch<React.SetStateAction<number>>;
  zoom: number;
  setZoom: React.Dispatch<React.SetStateAction<number>>;
  drawingColor: number | undefined;
  setDrawingColor: React.Dispatch<React.SetStateAction<number | undefined>>;
};

export const ViewEditorContext = React.createContext<ViewEditorContextValue>({
  view: {
    commands: [],
    description: '',
    loops: [],
  },
  viewWithCommandsApplied: {
    description: '',
    loops: [],
  },
  loopNumber: 0,
  setLoopNumber: () => {},
  celNumber: 0,
  setCelNumber: () => {},
  zoom: 6,
  setZoom: () => {},
  drawingColor: undefined,
  setDrawingColor: () => {},
});
