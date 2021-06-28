import { useContext, useEffect, useState } from 'react';
import { ViewEditorContext } from './ViewEditorContext';
import { ViewEditorControlContext } from './ViewEditorControlContext';
import { v4 } from 'uuid';

export function ViewEditorNavigationControls() {
  const {
    viewWithCommandsApplied,
    loopNumber,
    setLoopNumber,
    currentLoop,
    currentLoopCels,
    currentCel,
    celNumber,
    setCelNumber,
  } = useContext(ViewEditorContext);
  const { addCommands } = useContext(ViewEditorControlContext);
  const [animating, setAnimating] = useState(false);
  const [fps, setFps] = useState(5);

  useEffect(() => {
    if (animating && currentLoopCels.length > 1) {
      const interval = setInterval(() => {
        setCelNumber((prevCelNumber) =>
          prevCelNumber < currentLoopCels.length - 1 ? prevCelNumber + 1 : 0,
        );
      }, 1000 / fps);

      return () => {
        clearInterval(interval);
      };
    }
  }, [currentLoopCels, animating, fps, setCelNumber]);

  return (
    <div className="view-editor-navigation-controls">
      <section className="view-editor-nav-list">
        <h2>Loops</h2>
        <ul>
          {viewWithCommandsApplied.loops.map((loop, index) => (
            <li key={index} className={index === loopNumber ? 'current' : undefined}>
              <button
                type="button"
                className="secondary"
                onClick={() => {
                  addCommands([
                    {
                      type: 'SwapLoops',
                      a: loop.loopNumber,
                      b: loop.loopNumber - 1,
                      uuid: v4(),
                    },
                  ]);
                  setLoopNumber(loop.loopNumber - 1);
                }}
                disabled={loop.loopNumber < 1}
              >
                <i className="bi-chevron-up" role="img" aria-label="Move loop up" />
              </button>
              <button
                type="button"
                className="secondary"
                onClick={() => {
                  addCommands([
                    {
                      type: 'SwapLoops',
                      a: loop.loopNumber,
                      b: loop.loopNumber + 1,
                      uuid: v4(),
                    },
                  ]);
                  setLoopNumber(loop.loopNumber + 1);
                }}
                disabled={loop.loopNumber >= viewWithCommandsApplied.loops.length - 1}
              >
                <i className="bi-chevron-down" role="img" aria-label="Move loop down" />
              </button>
              <button
                type="button"
                className="item-number"
                onClick={() => {
                  setLoopNumber(index);
                }}
              >
                Loop {index}
                {loop.type === 'mirrored' && ` (mirrors loop ${loop.mirroredFromLoopNumber})`}
              </button>
              {loop.type === 'regular' && (
                <button
                  type="button"
                  className="secondary"
                  onClick={() =>
                    addCommands([
                      {
                        type: 'AddLoop',
                        newLoopNumber: loop.loopNumber + 1,
                        mirrorTargetNumber: loop.loopNumber,
                        uuid: v4(),
                      },
                    ])
                  }
                >
                  <i className="bi-plus-circle-dotted" role="img" aria-label="Create mirror" />
                </button>
              )}
              <button
                type="button"
                className="secondary"
                onClick={async () => {
                  if (await confirm('Are you sure you want to delete this loop?')) {
                    addCommands([{ type: 'DeleteLoop', loop: loop.loopNumber, uuid: v4() }]);
                  }
                }}
              >
                <i className="bi-trash" role="img" aria-label="Delete loop" />
              </button>
            </li>
          ))}
        </ul>
        <button
          type="button"
          onClick={() =>
            addCommands([
              {
                type: 'AddLoop',
                newLoopNumber: viewWithCommandsApplied.loops.length,
                uuid: v4(),
                mirrorTargetNumber: undefined,
              },
            ])
          }
        >
          Add loop
        </button>
      </section>

      <section className="view-editor-nav-list">
        <h2>Cels</h2>
        <ul>
          {currentLoopCels.map((cel, index) => (
            <li key={index} className={index === celNumber ? 'current' : undefined}>
              <button
                type="button"
                className="secondary"
                onClick={() => {
                  addCommands([
                    {
                      type: 'SwapCels',
                      loop:
                        currentLoop?.type === 'mirrored'
                          ? currentLoop.mirroredFromLoopNumber
                          : loopNumber,
                      a: cel.celNumber,
                      b: cel.celNumber - 1,
                      uuid: v4(),
                    },
                  ]);
                  setCelNumber(cel.celNumber - 1);
                }}
                disabled={cel.celNumber < 1}
              >
                <i className="bi-chevron-up" role="img" aria-label="Move cel up" />
              </button>
              <button
                type="button"
                className="secondary"
                onClick={() => {
                  addCommands([
                    {
                      type: 'SwapCels',
                      loop:
                        currentLoop?.type === 'mirrored'
                          ? currentLoop.mirroredFromLoopNumber
                          : loopNumber,
                      a: cel.celNumber,
                      b: cel.celNumber + 1,
                      uuid: v4(),
                    },
                  ]);
                  setCelNumber(cel.celNumber + 1);
                }}
                disabled={cel.celNumber >= currentLoopCels.length - 1}
              >
                <i className="bi-chevron-down" role="img" aria-label="Move cel down" />
              </button>
              <button
                type="button"
                className="item-number"
                onClick={() => {
                  setCelNumber(index);
                }}
              >
                Cel {index}
              </button>
              <button
                type="button"
                className="secondary"
                onClick={async () => {
                  if (await confirm('Are you sure you want to delete this cel?')) {
                    addCommands([{ type: 'DeleteCel', loop: loopNumber, cel: index, uuid: v4() }]);
                  }
                }}
              >
                <i className="bi-trash" role="img" aria-label="Delete cel" />
              </button>
            </li>
          ))}
        </ul>
        <button
          type="button"
          onClick={() =>
            addCommands([
              {
                type: 'AddCel',
                loop:
                  currentLoop?.type === 'mirrored'
                    ? currentLoop.mirroredFromLoopNumber
                    : loopNumber,
                newCelNumber: (currentCel?.celNumber ?? currentLoopCels.length - 1) + 1,
                uuid: v4(),
                width: currentCel?.width ?? 1,
                height: currentCel?.height ?? 1,
                transparentColor: currentCel?.transparentColor ?? 13,
              },
            ])
          }
        >
          Add cel
        </button>
      </section>

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
  );
}
