import { ColorPalette } from '../../ColorPalettes';
import { ViewLoop } from '../../Types/View';

export function renderViewCel(
  loop: ViewLoop,
  celNumber: number,
  palette: ColorPalette,
): Uint8Array {
  const cel = loop.cels[celNumber];
  const buffer = Buffer.alloc(cel.buffer.length * 4);

  for (let offset = 0; offset < cel.buffer.length; offset++) {
    const colorNumber = cel.buffer[offset];
    const color: [number, number, number, number] =
      colorNumber === cel.transparentColor ? [0, 0, 0, 0] : palette.colors[colorNumber];

    if (cel.mirrored) {
      const x = cel.width - 1 - (offset % cel.width);
      const y = Math.floor(offset / cel.width);
      const targetOffset = (x + y * cel.width) * 4;
      buffer.set(color, targetOffset);
    } else {
      buffer.set(color, offset * 4);
    }
  }

  return buffer;
}
