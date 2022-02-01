import React, { useContext, useMemo, useState } from 'react';
import { renderViewCel, EGAPalette } from '@agikit/core';
import { ViewCelCanvas } from './ViewCelCanvas';
import { CursorPosition } from './DrawingCanvas';
import { BrushStroke } from './ViewEditorBrushStrokes';
import { applyViewEditorCommands } from './ViewEditorCommands';
import { ViewEditorContext } from './ViewEditorContext';
import { ViewEditorControlContext } from './ViewEditorControlContext';
import { v4 } from 'uuid';
import { flipCelBuffer } from './EditingViewTypes';

export function ViewCelEditor() {
  const {
    currentCel,
    viewWithCommandsApplied,
    currentLoop,
    loopNumber,
    celNumber,
    currentLoopCels,
    drawingColor,
  } = useContext(ViewEditorContext);
  const { addCommands, zoom } = useContext(ViewEditorControlContext);
  const [currentBrushStroke, setCurrentBrushStroke] = useState<BrushStroke>();

  const viewWithCurrentBrushStrokeApplied = useMemo(
    () =>
      currentBrushStroke && currentCel
        ? applyViewEditorCommands(viewWithCommandsApplied, [
            {
              uuid: 'pending',
              loop:
                currentLoop?.type === 'mirrored' ? currentLoop.mirroredFromLoopNumber : loopNumber,
              cel: celNumber,
              type: 'Brush',
              brushStroke: currentBrushStroke,
            },
          ])
        : undefined,
    [currentCel, currentLoop, viewWithCommandsApplied, currentBrushStroke, celNumber, loopNumber],
  );

  const renderedCels = useMemo(() => {
    const currentLoopWithBrushStroke =
      viewWithCurrentBrushStrokeApplied?.loops[loopNumber] ?? currentLoop;
    let celsToRender = currentLoopCels;
    if (currentLoopWithBrushStroke?.type === 'mirrored') {
      const mirrorSourceLoop =
        viewWithCurrentBrushStrokeApplied?.loops[currentLoopWithBrushStroke.mirroredFromLoopNumber];
      if (mirrorSourceLoop?.type === 'regular') {
        celsToRender = mirrorSourceLoop.cels;
      }
    } else if (currentLoopWithBrushStroke?.type === 'regular') {
      celsToRender = currentLoopWithBrushStroke.cels;
    }

    return celsToRender.map((cel) => {
      const sourceBuffer =
        currentLoopWithBrushStroke?.type === 'mirrored' ? flipCelBuffer(cel) : cel.buffer;
      const renderedCel = renderViewCel(sourceBuffer, cel.transparentColor, EGAPalette);
      return { ...cel, buffer: renderedCel };
    });
  }, [currentLoopCels, currentLoop, loopNumber, viewWithCurrentBrushStrokeApplied]);

  const renderedCel = renderedCels[celNumber];

  const cursorDownInCanvas = (position: CursorPosition) => {
    if (currentCel && drawingColor != null) {
      const virtualPosition =
        currentLoop?.type === 'mirrored'
          ? { ...position, x: currentCel.width - position.x - 1 }
          : position;

      setCurrentBrushStroke({ drawingColor, positions: [virtualPosition] });
    }
  };

  const cursorMoveInCanvas = (position: CursorPosition) => {
    if (currentCel && drawingColor != null && currentBrushStroke) {
      const virtualPosition =
        currentLoop?.type === 'mirrored'
          ? { ...position, x: currentCel.width - position.x - 1 }
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
    if (currentCel && currentBrushStroke) {
      addCommands([
        {
          uuid: v4(),
          type: 'Brush',
          cel: celNumber,
          loop: currentLoop?.type === 'mirrored' ? currentLoop.mirroredFromLoopNumber : loopNumber,
          brushStroke: currentBrushStroke,
        },
      ]);
      setCurrentBrushStroke(undefined);
    }
  };

  if (!renderedCel) {
    return <></>;
  }

  return (
    <div className="view-editor-cel-canvas">
      <ViewCelCanvas
        cel={renderedCel}
        buffer={renderedCel.buffer}
        zoom={zoom}
        onCursorDown={cursorDownInCanvas}
        onCursorMove={cursorMoveInCanvas}
        onCursorUp={finishBrushStroke}
        onCursorOut={finishBrushStroke}
      />
    </div>
  );
}
