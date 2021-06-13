import { useMemo, useState } from 'react';

import { ViewLoopEditor } from './ViewLoopEditor';
import { EditingView, EditingViewLoop } from './EditingViewTypes';
import { ViewEditorContextValue, ViewEditorContext } from './ViewEditorContext';
import { applyViewEditorCommands, ViewEditorCommand } from './ViewEditorCommands';

export function ViewEditor({ view }: { view: EditingView }) {
  const [loopNumber, setLoopNumber] = useState(0);
  const [celNumber, setCelNumber] = useState(0);
  const [zoom, setZoom] = useState(6);
  const [drawingColor, setDrawingColor] = useState<number | undefined>(0);

  const viewWithCommandsApplied = useMemo(() => applyViewEditorCommands(view, view.commands), [
    view,
  ]);

  const contextValue = useMemo<ViewEditorContextValue>(
    () => ({
      view,
      viewWithCommandsApplied,
      celNumber,
      setCelNumber,
      loopNumber,
      setLoopNumber,
      zoom,
      setZoom,
      drawingColor,
      setDrawingColor,
    }),
    [view, viewWithCommandsApplied, celNumber, loopNumber, zoom, drawingColor],
  );

  const currentLoop = view.loops[loopNumber];

  return (
    <ViewEditorContext.Provider value={contextValue}>
      <div className="view-editor">
        <ViewLoopEditor />
        <div className="view-editor-navigation-controls">
          <h2>Loops</h2>
          <ul className="view-editor-loop-list">
            {view.loops.map((loop, index) => (
              <li key={index}>
                <button
                  type="button"
                  className={index === loopNumber ? 'current' : undefined}
                  onClick={() => {
                    setLoopNumber(index);
                    if (celNumber >= view.loops[index].cels.length) {
                      setCelNumber(0);
                    }
                  }}
                >
                  Loop {index}
                  {loop.type === 'mirrored' &&
                    ` (mirrors loop ${view.loops.indexOf(loop.mirroredFromLoop)})`}
                </button>
              </li>
            ))}
          </ul>

          <h2>Cels</h2>
          <ul className="view-editor-cel-list">
            {currentLoop.cels.map((cel: EditingViewLoop['cels'][number], index: number) => (
              <li key={index}>
                <button
                  type="button"
                  className={index === celNumber ? 'current' : undefined}
                  onClick={() => {
                    setCelNumber(index);
                  }}
                >
                  Cel {index}
                </button>
              </li>
            ))}
          </ul>
        </div>
      </div>
    </ViewEditorContext.Provider>
  );
}
