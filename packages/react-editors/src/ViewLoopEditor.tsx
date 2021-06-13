import React, { useEffect, useMemo, useState } from 'react';
import { AGIView, ViewCel } from 'agikit-core/dist/Types/View';
import { renderViewCel } from 'agikit-core/dist/Extract/View/RenderView';
import { EGAPalette } from 'agikit-core/dist/ColorPalettes';
import { ViewCelCanvas } from './ViewCelCanvas';
import { EditingView } from './EditingViewTypes';

export function ViewLoopEditor({
  view,
  loopNumber,
  celNumber,
  setCelNumber,
  zoom,
  setZoom,
}: {
  view: EditingView;
  loopNumber: number;
  celNumber: number;
  setCelNumber: React.Dispatch<React.SetStateAction<number>>;
  zoom: number;
  setZoom: React.Dispatch<React.SetStateAction<number>>;
}) {
  const loop = view.loops[loopNumber];
  const [animating, setAnimating] = useState(false);
  const [fps, setFps] = useState(5);
  const cel = loop.cels[celNumber];

  const renderedCels = useMemo(
    () => loop.cels.map((cel: ViewCel, index: number) => renderViewCel(loop, index, EGAPalette)),
    [loop],
  );

  useEffect(() => {
    if (animating) {
      const interval = setInterval(() => {
        setCelNumber((prevCelNumber) =>
          prevCelNumber < loop.cels.length - 1 ? prevCelNumber + 1 : 0,
        );
      }, 1000 / fps);

      return () => {
        clearInterval(interval);
      };
    }
  }, [loop.cels.length, animating, fps, setCelNumber]);

  return (
    <>
      <div className="view-editor-cel-canvas">
        {cel && (
          <ViewCelCanvas cel={loop.cels[celNumber]} buffer={renderedCels[celNumber]} zoom={zoom} />
        )}
      </div>
      <div className="view-editor-cel-controls">
        <div>
          Loop {loopNumber}, cel {celNumber}
          {cel && (
            <>
              <br />
              {cel.width}x{cel.height}
              <br />
              Transparent color: {cel.transparentColor}
              {cel.mirrored && (
                <>
                  <br />
                  Mirrored from loop{' '}
                  {view.loops.findIndex((otherLoop) => otherLoop === cel.mirroredFromLoop)}
                </>
              )}
            </>
          )}
        </div>
        <div>
          <button
            type="button"
            className="agikit-tool-button primary"
            title="Zoom out"
            disabled={zoom <= 1}
            onClick={() => setZoom((prevZoom) => prevZoom - 1)}
          >
            <i className="bi-zoom-out" role="img" aria-label="Zoom out" />
          </button>

          <button
            type="button"
            className="agikit-tool-button primary"
            title="Zoom in"
            onClick={() => setZoom((prevZoom) => prevZoom + 1)}
          >
            <i className="bi-zoom-in" role="img" aria-label="Zoom in" />
          </button>
        </div>
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
    </>
  );
}
