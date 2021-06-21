import { ColorPalette } from '../../ColorPalettes';
import { AGIView, ViewCel } from '../../Types/View';

export function renderViewCel(view: AGIView, cel: ViewCel, palette: ColorPalette): Uint8Array {
  const sourceCel = cel.mirrored ? view.loops[cel.mirroredFromLoopNumber].cels[cel.celNumber] : cel;
  if (sourceCel.mirrored) {
    throw new Error('Cel points at mirrored cel as mirror source');
  }

  const buffer = Buffer.alloc(sourceCel.buffer.length * 4);

  for (let offset = 0; offset < sourceCel.buffer.length; offset++) {
    const colorNumber = sourceCel.buffer[offset];
    const color: [number, number, number, number] =
      colorNumber === sourceCel.transparentColor ? [0, 0, 0, 0] : palette.colors[colorNumber];

    if (cel.mirrored) {
      const x = sourceCel.width - 1 - (offset % sourceCel.width);
      const y = Math.floor(offset / sourceCel.width);
      const targetOffset = (x + y * sourceCel.width) * 4;
      buffer.set(color, targetOffset);
    } else {
      buffer.set(color, offset * 4);
    }
  }

  return buffer;
}
