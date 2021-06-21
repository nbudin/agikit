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
import { buildEditingView } from './EditingViewTypes';

export function ViewLoopEditor() {
  const {
    view,
    viewWithCommandsApplied,
    loopNumber,
    celNumber,
    drawingColor,
    setDrawingColor,
    zoom,
    setZoom,
  } = useContext(ViewEditorContext);
  const { addCommands } = useContext(ViewEditorControlContext);
  const loop = viewWithCommandsApplied.loops[loopNumber];
  const cel = loop.cels[celNumber];
  const [currentBrushStroke, setCurrentBrushStroke] = useState<BrushStroke>();

  const viewWithCurrentBrushStrokeApplied = useMemo(
    () =>
      currentBrushStroke
        ? buildEditingView(
            applyViewEditorCommands(viewWithCommandsApplied, [
              {
                uuid: 'pending',
                loop: cel.mirrored ? cel.mirroredFromLoopNumber : loopNumber,
                cel: celNumber,
                type: 'Brush',
                brushStroke: currentBrushStroke,
              },
            ]),
          )
        : viewWithCommandsApplied,
    [cel, viewWithCommandsApplied, currentBrushStroke, celNumber, loopNumber],
  );

  const renderedCels = useMemo(
    () =>
      viewWithCurrentBrushStrokeApplied.loops[loopNumber].cels.map((cel: ViewCel) => {
        const renderedCel = renderViewCel(viewWithCurrentBrushStrokeApplied, cel, EGAPalette);
        return { ...cel, buffer: renderedCel };
      }),
    [viewWithCurrentBrushStrokeApplied, loopNumber],
  );

  const cursorDownInCanvas = (position: CursorPosition) => {
    if (drawingColor != null) {
      const virtualPosition = cel.mirrored
        ? { ...position, x: cel.width - position.x - 1 }
        : position;

      setCurrentBrushStroke({ drawingColor, positions: [virtualPosition] });
    }
  };

  const cursorMoveInCanvas = (position: CursorPosition) => {
    if (drawingColor != null && currentBrushStroke) {
      const virtualPosition = cel.mirrored
        ? { ...position, x: cel.width - position.x - 1 }
        : position;

      setCurrentBrushStroke((prevCurrentBrushStroke) => {
        const brushStrokeInProgress = prevCurrentBrushStroke || { drawingColor, positions: [] };
        if (
          !brushStrokeInProgress.positions.some(
            (existingPosition) =>
              existingPosition.x === virtualPosition.x && existingPosition.y === virtualPosition.y,
          )
        ) {
          return {
            ...brushStrokeInProgress,
            positions: [...brushStrokeInProgress.positions, virtualPosition],
          };
        }
        return brushStrokeInProgress;
      });
    }
  };

  const finishBrushStroke = () => {
    if (currentBrushStroke) {
      if (cel.mirrored) {
        addCommands([
          {
            uuid: v4(),
            type: 'Brush',
            cel: celNumber,
            loop: cel.mirroredFromLoopNumber,
            brushStroke: currentBrushStroke,
          },
        ]);
      }
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
              <br />
              {cel.mirrored ? (
                <>Mirrored from loop {cel.mirroredFromLoopNumber}</>
              ) : (
                <>TODO: figure out how to show what if anything is mirroring this cel</>
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
      </div>
    </>
  );
}
