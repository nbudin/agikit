import { useLayoutEffect, useRef, useState } from "react";
import { EGAPalette } from "agikit-core/dist/ColorPalettes";

export function PicCanvas({ buffer }: { buffer: Uint8Array }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [blobURL, setBlobURL] = useState("");

  useLayoutEffect(() => {
    if (!canvasRef.current) {
      return;
    }

    const ctx = canvasRef.current.getContext("2d");
    if (!ctx) {
      return;
    }

    for (let index = 0; index < buffer.length; index++) {
      const x = index % 160;
      const y = Math.floor(index / 160);
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

  return (
    <>
      <canvas
        ref={canvasRef}
        height="200"
        width="320"
        style={{ display: "none" }}
      ></canvas>
      <div
        className="pic-editor-canvas-display"
        style={{ backgroundImage: `url(${blobURL})` }}
      ></div>
    </>
  );
}
