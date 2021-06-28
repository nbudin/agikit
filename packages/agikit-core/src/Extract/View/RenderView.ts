import { ColorPalette } from '../../ColorPalettes';

export function renderViewCel(
  sourceBuffer: Uint8Array,
  transparentColor: number,
  palette: ColorPalette,
): Uint8Array {
  const buffer = Buffer.alloc(sourceBuffer.length * 4);

  for (let offset = 0; offset < sourceBuffer.length; offset++) {
    const colorNumber = sourceBuffer[offset];
    const color: [number, number, number, number] =
      colorNumber === transparentColor ? [0, 0, 0, 0] : palette.colors[colorNumber];

    buffer.set(color, offset * 4);
  }

  return buffer;
}
