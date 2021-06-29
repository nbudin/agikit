import React from 'react';
import { NonMirroredViewCel } from '@agikit/core/dist/Types/View';
import { EditingView, EditingViewLoop } from './EditingViewTypes';

export type ViewEditorContextValue = {
  view: EditingView;
  viewWithCommandsApplied: EditingView;
  loopNumber: number;
  setLoopNumber: React.Dispatch<React.SetStateAction<number>>;
  celNumber: number;
  setCelNumber: React.Dispatch<React.SetStateAction<number>>;
  currentLoop: EditingViewLoop | undefined;
  currentLoopCels: NonMirroredViewCel[];
  currentCel: NonMirroredViewCel | undefined;
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
  currentLoop: undefined,
  currentLoopCels: [],
  currentCel: undefined,
  drawingColor: undefined,
  setDrawingColor: () => {},
});
