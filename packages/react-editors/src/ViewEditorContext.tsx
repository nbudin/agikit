import React from 'react';
import { EditingView } from './EditingViewTypes';

export type ViewEditorContextValue = {
  view: EditingView;
  viewWithCommandsApplied: EditingView;
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
    commands: [],
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
