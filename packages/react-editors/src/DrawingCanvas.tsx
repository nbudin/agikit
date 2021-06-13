import { useLayoutEffect, useRef } from 'react';
import { renderCanvas2D, calculateLetterboxOffsets, calculateClampedPosition } from './rendering';

export type CursorPosition = {
  x: number;
  y: number;
};

type TypedRenderingContext2D = {
  type: '2d';
  ctx: CanvasRenderingContext2D;
};

type TypedRenderingContext = TypedRenderingContext2D;

export function DrawingCanvas({
  buffer,
  sourceWidth,
  sourceHeight,
  canvasWidth,
  canvasHeight,
  onCursorMove,
  onCursorDown,
  onCursorOut,
}: {
  buffer: Uint8Array;
  sourceWidth: number;
  sourceHeight: number;
  canvasWidth?: number;
  canvasHeight?: number;
  onCursorMove?: (position: CursorPosition) => void;
  onCursorDown?: (position: CursorPosition) => void;
  onCursorOut?: () => void;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const ctxRef = useRef<TypedRenderingContext | null>(null);

  useLayoutEffect(() => {
    if (!canvasRef.current) {
      return;
    }

    if (!ctxRef.current) {
      const canvas2Dctx = canvasRef.current.getContext('2d');
      if (canvas2Dctx) {
        ctxRef.current = {
          type: '2d',
          ctx: canvas2Dctx,
        };
      } else {
        return;
      }
    }

    renderCanvas2D(canvasRef.current, ctxRef.current.ctx, sourceWidth, sourceHeight, buffer);
  }, [buffer, sourceWidth, sourceHeight, canvasWidth, canvasHeight]);

  const calculateCursorPosition = (event: React.MouseEvent) => {
    if (!canvasRef.current) {
      return;
    }

    const { horizontalOffset, verticalOffset } = calculateLetterboxOffsets(
      sourceWidth * 2,
      sourceHeight,
      canvasRef.current.offsetWidth,
      canvasRef.current.offsetHeight,
    );

    if (
      event.clientX < canvasRef.current.offsetLeft + horizontalOffset ||
      event.clientX >
        canvasRef.current.offsetLeft + canvasRef.current.offsetWidth - horizontalOffset ||
      event.clientY < canvasRef.current.offsetTop + verticalOffset ||
      event.clientY > canvasRef.current.offsetTop + canvasRef.current.offsetHeight - verticalOffset
    ) {
      return;
    }

    const x = calculateClampedPosition(
      event.clientX,
      canvasRef.current.offsetLeft + horizontalOffset,
      canvasRef.current.offsetWidth - horizontalOffset * 2,
      sourceWidth,
    );
    const y = calculateClampedPosition(
      event.clientY,
      canvasRef.current.offsetTop + verticalOffset,
      canvasRef.current.offsetHeight - verticalOffset * 2,
      sourceHeight,
    );

    return { x, y };
  };

  const mouseMoved = (event: React.MouseEvent) => {
    if (!canvasRef.current || !(onCursorMove || onCursorOut)) {
      return;
    }

    const position = calculateCursorPosition(event);

    if (position && onCursorMove) {
      onCursorMove(position);
    } else if (onCursorOut) {
      onCursorOut();
    }
  };

  const mouseDown = (event: React.MouseEvent) => {
    if (!canvasRef.current || !onCursorDown) {
      return;
    }

    const position = calculateCursorPosition(event);

    if (position) {
      onCursorDown(position);
    }
  };

  return (
    <canvas
      ref={canvasRef}
      className="pic-editor-canvas-display"
      width={canvasWidth}
      height={canvasHeight}
      onMouseMove={mouseMoved}
      onMouseDown={mouseDown}
      onMouseOut={() => {
        if (onCursorOut) {
          onCursorOut();
        }
      }}
    />
  );
}
