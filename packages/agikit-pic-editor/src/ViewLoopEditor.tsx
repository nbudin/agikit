import React, { useEffect, useMemo, useState } from 'react';
import { AGIView } from 'agikit-core/dist/Types/View';
import { ViewCelCanvas } from './ViewCelCanvas';
import { renderViewCel } from 'agikit-core/dist/Extract/View/RenderView';
import { EGAPalette } from 'agikit-core/dist/ColorPalettes';

export function ViewLoopEditor({ view, loopNumber }: { view: AGIView; loopNumber: number }) {
  const loop = view.loops[loopNumber];
  const [celNumber, setCelNumber] = useState(0);
  const [animating, setAnimating] = useState(false);
  const [fps, setFps] = useState(5);
  const cel = loop.cels[celNumber];

  const renderedCels = useMemo(
    () => loop.cels.map((cel, index) => renderViewCel(loop, index, EGAPalette)),
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
  }, [loop.cels.length, animating, fps]);

  return (
    <>
      Loop {loopNumber}
      {cel && (
        <>
          <br />
          Cel {celNumber}
          <br />
          <ViewCelCanvas cel={loop.cels[celNumber]} buffer={renderedCels[celNumber]} zoom={6} />
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
      <div>
        <label>
          <input
            type="checkbox"
            checked={animating}
            onChange={(event) => setAnimating(event.target.checked)}
          />{' '}
          Animate
        </label>
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
    </>
  );
}
