import { useLayoutEffect, useRef } from "react";
import { EGAPalette } from "agikit-core/dist/ColorPalettes";

export function PicCanvas({ buffer }: { buffer: Uint8Array }) {
  const ref = useRef<HTMLCanvasElement>(null);
  useLayoutEffect(() => {
    if (!ref.current) {
      return;
    }

    const ctx = ref.current.getContext("2d");
    if (!ctx) {
      return;
    }

    for (let index = 0; index < buffer.length; index++) {
      const x = index % 160;
      const y = Math.floor(index / 160);
      const color = EGAPalette[buffer[index]];
      ctx.fillStyle = color;
      ctx.fillRect(x * 4, y * 2, 4, 2);
    }
  }, [buffer]);

  return <canvas width="640" height="400" ref={ref}></canvas>;
}
