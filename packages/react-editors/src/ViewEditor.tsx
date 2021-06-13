import { useMemo, useState } from 'react';
import { AGIView } from 'agikit-core/dist/Types/View';

import { ViewLoopEditor } from './ViewLoopEditor';
import { EditingView, EditingViewLoop } from './EditingViewTypes';
import { ViewEditorContextValue, ViewEditorContext } from './ViewEditorContext';
import { ViewEditorCommand } from './ViewEditorCommands';

export function ViewEditor({ view }: { view: EditingView }) {
  const [loopNumber, setLoopNumber] = useState(0);
  const [celNumber, setCelNumber] = useState(0);
  const [zoom, setZoom] = useState(6);
  const [drawingColor, setDrawingColor] = useState<number | undefined>(0);
  const [commands, setCommands] = useState<ViewEditorCommand[]>([]);

  const contextValue = useMemo<ViewEditorContextValue>(
    () => ({
      view,
      celNumber,
      setCelNumber,
      loopNumber,
      setLoopNumber,
      zoom,
      setZoom,
      commands,
      setCommands,
      drawingColor,
      setDrawingColor,
    }),
    [view, celNumber, loopNumber, zoom, commands, drawingColor],
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
