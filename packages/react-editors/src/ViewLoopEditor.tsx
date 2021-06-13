import { useContext, useEffect, useMemo, useState } from 'react';
import { ViewCel } from 'agikit-core/dist/Types/View';
import { renderViewCel } from 'agikit-core/dist/Extract/View/RenderView';
import { EGAPalette } from 'agikit-core/dist/ColorPalettes';
import { ViewCelCanvas } from './ViewCelCanvas';
import ColorSelector from './ColorSelector';
import { CursorPosition } from './DrawingCanvas';
import { BrushStroke } from './ViewEditorBrushStrokes';
import { applyViewEditorCommands } from './ViewEditorCommands';
import { ViewEditorContext } from './ViewEditorContext';
import { ViewEditorControlContext } from './ViewEditorControlContext';
import { v4 } from 'uuid';

export function ViewLoopEditor() {
  const {
    view,
    viewWithCommandsApplied,
    loopNumber,
    celNumber,
    setCelNumber,
    drawingColor,
    setDrawingColor,
    zoom,
    setZoom,
  } = useContext(ViewEditorContext);
  const { addCommands } = useContext(ViewEditorControlContext);
  const loop = viewWithCommandsApplied.loops[loopNumber];
  const [animating, setAnimating] = useState(false);
  const [fps, setFps] = useState(5);
  const cel = loop.cels[celNumber];
  const [currentBrushStroke, setCurrentBrushStroke] = useState<BrushStroke>();

  const viewWithCurrentBrushStrokeApplied = useMemo(
    () =>
      currentBrushStroke
        ? applyViewEditorCommands(viewWithCommandsApplied, [
            {
              uuid: 'pending',
              loop: loopNumber,
              cel: celNumber,
              type: 'Brush',
              brushStroke: currentBrushStroke,
            },
          ])
        : viewWithCommandsApplied,
    [viewWithCommandsApplied, currentBrushStroke, celNumber, loopNumber],
  );

  const renderedCels = useMemo(
    () =>
      viewWithCurrentBrushStrokeApplied.loops[loopNumber].cels.map((cel: ViewCel) => {
        const renderedCel = renderViewCel(cel, EGAPalette);
        return { ...cel, buffer: renderedCel };
      }),
    [viewWithCurrentBrushStrokeApplied, loopNumber],
  );

  const cursorDownInCanvas = (position: CursorPosition) => {
    if (drawingColor != null) {
      setCurrentBrushStroke({ drawingColor, positions: [position] });
    }
  };

  const cursorMoveInCanvas = (position: CursorPosition) => {
    if (drawingColor != null && currentBrushStroke) {
      setCurrentBrushStroke((prevCurrentBrushStroke) => {
        const brushStrokeInProgress = prevCurrentBrushStroke || { drawingColor, positions: [] };
        if (
          !brushStrokeInProgress.positions.some(
            (existingPosition) =>
              existingPosition.x === position.x && existingPosition.y === position.y,
          )
        ) {
          return {
            ...brushStrokeInProgress,
            positions: [...brushStrokeInProgress.positions, position],
          };
        }
        return brushStrokeInProgress;
      });
    }
  };

  const finishBrushStroke = () => {
    if (currentBrushStroke) {
      addCommands([
        {
          uuid: v4(),
          type: 'Brush',
          cel: celNumber,
          loop: loopNumber,
          brushStroke: currentBrushStroke,
        },
      ]);
      setCurrentBrushStroke(undefined);
    }
  };

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
          <ViewCelCanvas
            cel={renderedCels[celNumber]}
            buffer={renderedCels[celNumber].buffer}
            zoom={zoom}
            onCursorDown={cursorDownInCanvas}
            onCursorMove={cursorMoveInCanvas}
            onCursorUp={finishBrushStroke}
            onCursorOut={finishBrushStroke}
          />
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
        <div className="view-editor-tools">
          <button
            type="button"
            className="agikit-tool-button secondary"
            title="Zoom out"
            disabled={zoom <= 1}
            onClick={() => setZoom((prevZoom) => prevZoom - 1)}
          >
            <i className="bi-zoom-out" role="img" aria-label="Zoom out" />
          </button>

          <button
            type="button"
            className="agikit-tool-button secondary"
            title="Zoom in"
            onClick={() => setZoom((prevZoom) => prevZoom + 1)}
          >
            <i className="bi-zoom-in" role="img" aria-label="Zoom in" />
          </button>

          <ColorSelector
            palette={EGAPalette}
            color={drawingColor}
            setColor={setDrawingColor}
            transparentColor={cel.transparentColor}
          />
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
