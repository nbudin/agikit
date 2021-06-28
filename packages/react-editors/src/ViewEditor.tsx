import { useEffect, useMemo, useState } from 'react';

import { EditingView } from './EditingViewTypes';
import { ViewEditorContextValue, ViewEditorContext } from './ViewEditorContext';
import { applyViewEditorCommands } from './ViewEditorCommands';
import { ViewEditorNavigationControls } from './ViewEditorNavigationControls';
import { ViewCelEditor } from './ViewCelEditor';
import { ViewEditorCelControls } from './ViewEditorCelControls';

export function ViewEditor({ view }: { view: EditingView }) {
  const [loopNumber, setLoopNumber] = useState(0);
  const [celNumber, setCelNumber] = useState(0);
  const [zoom, setZoom] = useState(6);
  const [drawingColor, setDrawingColor] = useState<number | undefined>(0);

  const viewWithCommandsApplied = useMemo(() => applyViewEditorCommands(view, view.commands), [
    view,
  ]);

  const currentLoop = useMemo(() => viewWithCommandsApplied.loops[loopNumber], [
    viewWithCommandsApplied,
    loopNumber,
  ]);
  const currentLoopCels = useMemo(() => {
    if (currentLoop?.type === 'mirrored') {
      const mirrorSourceLoop = viewWithCommandsApplied.loops[currentLoop.mirroredFromLoopNumber];
      if (!mirrorSourceLoop) {
        throw new Error(
          `Loop ${currentLoop.loopNumber} specifies nonexistent loop ${currentLoop.mirroredFromLoopNumber} as mirror source`,
        );
      }
      if (mirrorSourceLoop.type === 'mirrored') {
        throw new Error(
          `Loop ${currentLoop.loopNumber} specifies mirrored loop ${mirrorSourceLoop.loopNumber} as mirror source`,
        );
      }
      return mirrorSourceLoop.cels;
    } else if (currentLoop) {
      return currentLoop.cels;
    }
    return [];
  }, [viewWithCommandsApplied, currentLoop]);

  useEffect(() => {
    if (celNumber > currentLoopCels.length && celNumber > 0) {
      setCelNumber(0);
    }
  }, [celNumber, currentLoopCels]);

  const currentCel = currentLoopCels[celNumber];

  const contextValue = useMemo<ViewEditorContextValue>(
    () => ({
      view,
      viewWithCommandsApplied,
      celNumber,
      setCelNumber,
      loopNumber,
      setLoopNumber,
      currentLoop,
      currentLoopCels,
      currentCel,
      zoom,
      setZoom,
      drawingColor,
      setDrawingColor,
    }),
    [
      view,
      viewWithCommandsApplied,
      currentLoop,
      currentLoopCels,
      currentCel,
      celNumber,
      loopNumber,
      zoom,
      drawingColor,
    ],
  );

  return (
    <ViewEditorContext.Provider value={contextValue}>
      <div className="view-editor">
        <ViewCelEditor />
        <ViewEditorCelControls />
        <ViewEditorNavigationControls />
      </div>
    </ViewEditorContext.Provider>
  );
}
