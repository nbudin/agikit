import { AGIView, ViewLoop, ViewCel } from '../../Types/View';

export function readViewResource(data: Buffer): AGIView {
  // AGI Spec says the purpose of the first 2 bytes is unknown :/
  // http://agiwiki.sierrahelp.com/index.php?title=AGI_Specifications:_Chapter_8_-_View_Resources#ss8.1
  let offset = 2;

  const consumeUInt8 = () => {
    const value = data.readUInt8(offset);
    offset += 1;
    return value;
  };

  const consumeUInt16LE = () => {
    const value = data.readUInt16LE(offset);
    offset += 2;
    return value;
  };

  const consumeNullTerminatedString = () => {
    const chars: number[] = [];
    let char: number;
    do {
      char = consumeUInt8();
      if (char > 0) {
        chars.push(char);
      }
    } while (char > 0);
    return Buffer.from(chars).toString('ascii');
  };

  const loopCount = consumeUInt8();
  const descriptionOffset = consumeUInt16LE();
  const loopOffsets: number[] = [];
  for (let loopNumber = 0; loopNumber < loopCount; loopNumber++) {
    loopOffsets.push(consumeUInt16LE());
  }

  let description: string | undefined;
  if (descriptionOffset > 0) {
    offset = descriptionOffset;
    description = consumeNullTerminatedString();
  }

  const loops: ViewLoop[] = [];
  for (let loopNumber = 0; loopNumber < loopCount; loopNumber++) {
    const loopOffset = loopOffsets[loopNumber];
    offset = loopOffset;
    const celCount = consumeUInt8();
    const celOffsets: number[] = [];
    for (let celNumber = 0; celNumber < celCount; celNumber++) {
      celOffsets.push(loopOffset + consumeUInt16LE());
    }

    const loop: ViewLoop = {
      loopNumber,
      cels: [],
    };

    for (let celNumber = 0; celNumber < celCount; celNumber++) {
      const celOffset = celOffsets[celNumber];
      offset = celOffset;
      const width = consumeUInt8();
      const height = consumeUInt8();
      const transparencyMirroringByte = consumeUInt8();
      const transparentColor = transparencyMirroringByte & 0x0f;
      const mirrored = (transparencyMirroringByte & 0b10000000) > 0;
      const mirroredFromLoopNumber = (transparencyMirroringByte & 0b01110000) >> 4;

      if (mirrored && mirroredFromLoopNumber !== loopNumber) {
        loop.cels.push({
          celNumber,
          width,
          height,
          transparentColor,
          mirrored,
          mirroredFromLoopNumber,
        });
      } else {
        const pixels: number[] = [];

        for (let y = 0; y < height; y++) {
          let x = 0;

          let byte: number;
          do {
            byte = consumeUInt8();
            if (byte > 0) {
              const color = byte >> 4;
              const pixelCount = byte & 0x0f;
              x += pixelCount;
              for (let pixelNumber = 0; pixelNumber < pixelCount; pixelNumber++) {
                pixels.push(color);
              }
            }
          } while (byte > 0);

          if (x < width) {
            const fillPixels = width - x;
            for (let pixelNumber = 0; pixelNumber < fillPixels; pixelNumber++) {
              pixels.push(transparentColor);
            }
          }
        }

        loop.cels.push({
          celNumber,
          width,
          height,
          transparentColor,
          buffer: Uint8Array.from(pixels),
          mirrored: false,
          mirroredFromLoopNumber: undefined,
        });
      }
    }

    loops.push(loop);
  }

  return {
    description,
    loops,
  };
}
