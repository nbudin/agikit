import { useEffect, useMemo, useState } from 'react';

import { ViewLoopEditor } from './ViewLoopEditor';
import { buildEditingView, EditingView, EditingViewLoop } from './EditingViewTypes';
import { ViewEditorContextValue, ViewEditorContext } from './ViewEditorContext';
import { applyViewEditorCommands, ViewEditorCommand } from './ViewEditorCommands';

export function ViewEditor({ view }: { view: EditingView }) {
  const [loopNumber, setLoopNumber] = useState(0);
  const [celNumber, setCelNumber] = useState(0);
  const [zoom, setZoom] = useState(6);
  const [drawingColor, setDrawingColor] = useState<number | undefined>(0);

  const viewWithCommandsApplied = useMemo(
    () => buildEditingView(applyViewEditorCommands(view, view.commands)),
    [view],
  );

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

  const [animating, setAnimating] = useState(false);
  const [fps, setFps] = useState(5);
  const currentLoop = view.loops[loopNumber];

  useEffect(() => {
    if (animating && currentLoop) {
      const interval = setInterval(() => {
        setCelNumber((prevCelNumber) =>
          prevCelNumber < currentLoop.cels.length - 1 ? prevCelNumber + 1 : 0,
        );
      }, 1000 / fps);

      return () => {
        clearInterval(interval);
      };
    }
  }, [currentLoop, animating, fps, setCelNumber]);

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
                    const targetLoop = view.loops[index];
                    if (!targetLoop || celNumber >= targetLoop.cels.length) {
                      setCelNumber(0);
                    }
                  }}
                >
                  Loop {index}
                  {loop.type === 'mirrored' && ` (mirrors loop ${loop.mirroredFromLoopNumber})`}
                </button>
              </li>
            ))}
          </ul>

          <h2>Cels</h2>
          <ul className="view-editor-cel-list">
            {currentLoop?.cels.map((cel: EditingViewLoop['cels'][number], index: number) => (
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

          <div>
            <button
              type="button"
              className="agikit-tool-button primary"
              title={animating ? 'Pause animation' : 'Play animation'}
              onClick={() => setAnimating((prevAnimating) => !prevAnimating)}
            >
              <i
                className={animating ? 'bi-pause' : 'bi-play'}
                role="img"
                aria-label={animating ? 'Pause animation' : 'Play animation'}
              />
            </button>
            <label>
              <input
                type="range"
                value={fps}
                onChange={(event) => setFps(event.target.valueAsNumber)}
                min={0.5}
                max={60}
                step={0.5}
              />{' '}
              {fps} FPS
            </label>
          </div>
        </div>
      </div>
    </ViewEditorContext.Provider>
  );
}
