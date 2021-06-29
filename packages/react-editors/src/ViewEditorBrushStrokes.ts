import { ViewCel } from '@agikit/core/dist/Types/View';
import { CursorPosition } from './DrawingCanvas';

export type BrushStroke = {
  drawingColor: number;
  positions: CursorPosition[];
};

export function applyBrushStroke(
  buffer: Uint8Array,
  cel: ViewCel,
  brushStroke: BrushStroke,
): Uint8Array {
  const newBuffer = new Uint8Array(buffer);
  brushStroke.positions.forEach((position) => {
    newBuffer[position.x + position.y * cel.width] = brushStroke.drawingColor;
  });
  return newBuffer;
}

export function applyBrushStrokes(
  buffer: Uint8Array,
  cel: ViewCel,
  brushStrokes: BrushStroke[],
): Uint8Array {
  return brushStrokes.reduce(
    (workingBuffer, brushStroke) => applyBrushStroke(workingBuffer, cel, brushStroke),
    buffer,
  );
}
