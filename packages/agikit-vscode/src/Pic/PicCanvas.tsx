import React, { useLayoutEffect, useRef, useState } from 'react';
import { EGAPalette } from 'agikit-core/dist/ColorPalettes';

const PIC_WIDTH = 160;
const PIC_HEIGHT = 200;
const DISPLAY_ASPECT_RATIO = (PIC_WIDTH * 2) / PIC_HEIGHT;

function clamp(n: number, min: number, max: number): number {
  if (n < min) {
    return min;
  }

  if (n > max) {
    return max;
  }

  return n;
}

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

export type CursorPosition = {
  x: number;
  y: number;
};

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
  const imgRef = useRef<HTMLImageElement>(null);
  const [blobURL, setBlobURL] = useState('');

  useLayoutEffect(() => {
    if (!canvasRef.current) {
      return;
    }

    const ctx = canvasRef.current.getContext('2d');
    if (!ctx) {
      return;
    }

    for (let index = 0; index < buffer.length; index++) {
      const x = index % PIC_WIDTH;
      const y = Math.floor(index / PIC_WIDTH);
      const color = EGAPalette[buffer[index]];
      ctx.fillStyle = color;
      ctx.fillRect(x * 2, y, 2, 1);
    }

    canvasRef.current.toBlob((blob) => {
      if (blob) {
        setBlobURL((prevURL) => {
          if (prevURL) {
            window.requestAnimationFrame(() => {
              URL.revokeObjectURL(prevURL);
            });
          }
          return URL.createObjectURL(blob);
        });
      }
    });
  }, [buffer]);

  const calculateCursorPosition = (event: React.MouseEvent) => {
    if (!imgRef.current) {
      return;
    }

    const elementAspectRatio = imgRef.current.offsetWidth / imgRef.current.offsetHeight;
    let horizontalOffset = 0;
    let verticalOffset = 0;
    if (elementAspectRatio < DISPLAY_ASPECT_RATIO) {
      // it's letterboxed on the top and bottom
      const imageHeight = imgRef.current.offsetWidth / DISPLAY_ASPECT_RATIO;
      verticalOffset = (imgRef.current.offsetHeight - imageHeight) / 2;
    } else if (elementAspectRatio > DISPLAY_ASPECT_RATIO) {
      // it's letterboxed on the left and right
      const imageWidth = imgRef.current.offsetHeight * DISPLAY_ASPECT_RATIO;
      horizontalOffset = (imgRef.current.offsetWidth - imageWidth) / 2;
    }

    if (
      event.clientX < imgRef.current.offsetLeft + horizontalOffset ||
      event.clientX > imgRef.current.offsetLeft + imgRef.current.offsetWidth - horizontalOffset ||
      event.clientY < imgRef.current.offsetTop + verticalOffset ||
      event.clientY > imgRef.current.offsetTop + imgRef.current.offsetHeight - verticalOffset
    ) {
      return;
    }

    const x = calculateClampedPosition(
      event.clientX,
      imgRef.current.offsetLeft + horizontalOffset,
      imgRef.current.offsetWidth - horizontalOffset * 2,
      PIC_WIDTH,
    );
    const y = calculateClampedPosition(
      event.clientY,
      imgRef.current.offsetTop + verticalOffset,
      imgRef.current.offsetHeight - verticalOffset * 2,
      PIC_HEIGHT,
    );

    return { x, y };
  };

  const mouseMoved = (event: React.MouseEvent) => {
    if (!imgRef.current) {
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
    if (!imgRef.current) {
      return;
    }

    const position = calculateCursorPosition(event);

    if (position) {
      onCursorDown(position);
    }
  };

  return (
    <>
      <canvas
        ref={canvasRef}
        height={PIC_HEIGHT}
        width={PIC_WIDTH * 2}
        style={{ display: 'none' }}
      ></canvas>
      <img
        ref={imgRef}
        src={blobURL}
        className="pic-editor-canvas-display"
        onMouseMove={mouseMoved}
        onMouseDown={mouseDown}
        onMouseOut={() => {
          onCursorOut();
        }}
      />
    </>
  );
}
