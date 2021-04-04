import React, { useLayoutEffect, useMemo, useRef, useState } from 'react';
import { EGAPalette } from 'agikit-core/dist/ColorPalettes';
import { clamp } from 'lodash';

const PIC_WIDTH = 160;
const PIC_HEIGHT = 200;
const DISPLAY_ASPECT_RATIO = (PIC_WIDTH * 2) / PIC_HEIGHT;

function calculateClampedPosition(
  clientPosition: number,
  offsetStart: number,
  offsetSize: number,
  virtualSize: number,
): number {
  const offsetPosition = clientPosition - offsetStart;
  const fractionalPosition = offsetPosition / offsetSize;
  return clamp(Math.round(fractionalPosition * virtualSize), 0, virtualSize - 1);
}

function calculateLetterboxOffsets(width: number, height: number) {
  const elementAspectRatio = width / height;
  let horizontalOffset = 0;
  let verticalOffset = 0;
  let imageWidth = width;
  let imageHeight = height;
  if (elementAspectRatio < DISPLAY_ASPECT_RATIO) {
    // it's letterboxed on the top and bottom
    imageHeight = width / DISPLAY_ASPECT_RATIO;
    verticalOffset = (height - imageHeight) / 2;
  } else if (elementAspectRatio > DISPLAY_ASPECT_RATIO) {
    // it's letterboxed on the left and right
    imageWidth = height * DISPLAY_ASPECT_RATIO;
    horizontalOffset = (width - imageWidth) / 2;
  }

  return { horizontalOffset, verticalOffset, imageWidth, imageHeight };
}

function renderCanvas2D(
  canvas: HTMLCanvasElement,
  ctx: CanvasRenderingContext2D,
  buffer: Uint8Array,
) {
  const { imageWidth, imageHeight, horizontalOffset, verticalOffset } = calculateLetterboxOffsets(
    canvas.offsetWidth,
    canvas.offsetHeight,
  );

  ctx.canvas.width = canvas.offsetWidth;
  ctx.canvas.height = canvas.offsetHeight;
  ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
  // const xRatio = imageWidth / PIC_WIDTH;
  // const yRatio = imageHeight / PIC_HEIGHT;
  const bufferCanvas = new OffscreenCanvas(160, 200);
  const bufferCanvasCtx = bufferCanvas.getContext('2d');
  if (bufferCanvasCtx) {
    const imageData = bufferCanvasCtx.getImageData(0, 0, 160, 200);
    imageData.data.set(buffer);
    bufferCanvasCtx.putImageData(imageData, 0, 0);

    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(bufferCanvas, horizontalOffset, verticalOffset, imageWidth, imageHeight);
  }
  // const imageData = ctx.getImageData(horizontalOffset, verticalOffset, imageWidth, imageHeight);

  // for (let imageY = 0; imageY < imageData.height; imageY++) {
  //   for (let imageX = 0; imageX < imageData.width; imageX++) {
  //     const pixelX = clamp(Math.round(imageX / xRatio), 0, PIC_WIDTH - 1);
  //     const pixelY = clamp(Math.round(imageY / yRatio), 0, PIC_HEIGHT - 1);
  //     const pixelIndex = (pixelY * PIC_WIDTH + pixelX) * EGAPalette.bpp;
  //     const imageDataOffset = (imageY * imageData.width + imageX) * 4;
  //     imageData.data.set(buffer.subarray(pixelIndex, pixelIndex + EGAPalette.bpp), imageDataOffset);
  //     imageData.data[imageDataOffset + 3] = 255;
  //   }
  // }

  // ctx.putImageData(imageData, horizontalOffset, verticalOffset);
}

function renderCanvasGL(
  canvas: HTMLCanvasElement,
  ctx: WebGL2RenderingContext,
  buffer: Uint8Array,
) {
  ctx.clearColor(1.0, 0.0, 1.0, 1.0);
  ctx.clear(ctx.COLOR_BUFFER_BIT);
}

export type CursorPosition = {
  x: number;
  y: number;
};

type TypedRenderingContextWebGL2 = {
  type: 'webgl2';
  ctx: WebGL2RenderingContext;
};

type TypedRenderingContext2D = {
  type: '2d';
  ctx: CanvasRenderingContext2D;
};

type TypedRenderingContext = TypedRenderingContext2D | TypedRenderingContextWebGL2;

export function PicCanvas({
  buffer,
  onCursorMove,
  onCursorDown,
  onCursorOut,
}: {
  buffer: Uint8Array;
  onCursorMove: (position: CursorPosition) => void;
  onCursorDown: (position: CursorPosition) => void;
  onCursorOut: () => void;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const ctxRef = useRef<TypedRenderingContext | null>(null);

  useLayoutEffect(() => {
    if (!canvasRef.current) {
      return;
    }

    if (!ctxRef.current) {
      // const webGL2ctx = canvasRef.current.getContext('webgl2');
      // if (webGL2ctx) {
      //   ctxRef.current = {
      //     type: 'webgl2',
      //     ctx: webGL2ctx,
      //   };
      // } else {
      const canvas2Dctx = canvasRef.current.getContext('2d');
      if (canvas2Dctx) {
        ctxRef.current = {
          type: '2d',
          ctx: canvas2Dctx,
        };
      } else {
        return;
      }
      // }
    }

    if (ctxRef.current.type === 'webgl2') {
      renderCanvasGL(canvasRef.current, ctxRef.current.ctx, buffer);
    } else {
      renderCanvas2D(canvasRef.current, ctxRef.current.ctx, buffer);
    }
  }, [buffer]);

  const calculateCursorPosition = (event: React.MouseEvent) => {
    if (!canvasRef.current) {
      return;
    }

    const { horizontalOffset, verticalOffset } = calculateLetterboxOffsets(
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
      PIC_WIDTH,
    );
    const y = calculateClampedPosition(
      event.clientY,
      canvasRef.current.offsetTop + verticalOffset,
      canvasRef.current.offsetHeight - verticalOffset * 2,
      PIC_HEIGHT,
    );

    return { x, y };
  };

  const mouseMoved = (event: React.MouseEvent) => {
    if (!canvasRef.current) {
      return;
    }

    const position = calculateCursorPosition(event);

    if (position) {
      onCursorMove(position);
    } else {
      onCursorOut();
    }
  };

  const mouseDown = (event: React.MouseEvent) => {
    if (!canvasRef.current) {
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
      onMouseMove={mouseMoved}
      onMouseDown={mouseDown}
      onMouseOut={() => {
        onCursorOut();
      }}
    />
  );
}
