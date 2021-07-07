import { AGIView, NonMirroredViewCel, ViewLoop } from '../Types/View';
import { encodeUInt16LE } from '../DataEncoding';

function buildHeaderForOptionalBuffers(
  buffers: (Buffer | undefined)[],
  startingOffset: number,
): Buffer {
  const header: number[] = [];
  let offset = startingOffset;
  const bufferOffsets = new Map<Buffer, number>();
  buffers.forEach((buffer) => {
    if (buffer == null) {
      header.push(...encodeUInt16LE(0));
    } else {
      const existingOffset = bufferOffsets.get(buffer);
      if (existingOffset != null) {
        header.push(...encodeUInt16LE(existingOffset));
      } else {
        bufferOffsets.set(buffer, offset);
        header.push(...encodeUInt16LE(offset));
        offset += buffer.byteLength;
      }
    }
  });

  return Buffer.from(header);
}

function concatOptionalBuffers(buffers: (Buffer | undefined)[]): Buffer {
  const definedBuffers: Buffer[] = [];
  const seenBuffers = new Set<Buffer>();
  buffers.forEach((buffer) => {
    if (buffer != null && !seenBuffers.has(buffer)) {
      seenBuffers.add(buffer);
      definedBuffers.push(buffer);
    }
  });
  return Buffer.concat(definedBuffers);
}

function encodeCel(cel: NonMirroredViewCel, mirrorSourceLoopNumber: number | undefined): Buffer {
  let transparencyMirroringByte = cel.transparentColor & 0x0f;
  if (mirrorSourceLoopNumber != null) {
    transparencyMirroringByte += 0b10000000;
    transparencyMirroringByte += (mirrorSourceLoopNumber & 0b111) << 4;
  }

  const data = [cel.width, cel.height, transparencyMirroringByte];
  for (let y = 0; y < cel.height; y++) {
    let lastPixelColor: number | undefined;
    let runLength = 0;
    for (let x = 0; x < cel.width; x++) {
      const offset = x + y * cel.width;
      const pixelColor = cel.buffer[offset];

      if (lastPixelColor == null) {
        lastPixelColor = pixelColor;
      } else if (lastPixelColor !== pixelColor) {
        data.push(((lastPixelColor & 0b1111) << 4) + (runLength & 0b1111));
        runLength = 0;
        lastPixelColor = pixelColor;
      }

      runLength += 1;
    }

    if (lastPixelColor != null && lastPixelColor !== cel.transparentColor) {
      data.push(((lastPixelColor & 0b1111) << 4) + (runLength & 0b1111));
    }

    data.push(0);
  }

  return Buffer.from(data);
}

function encodeLoop(loop: ViewLoop, mirrorSourceLoopNumber: number | undefined): Buffer {
  const encodedCels = loop.cels.map((cel) => {
    if (cel.mirrored) {
      throw new Error(`Can't encode mirrored cel ${cel.celNumber} in loop ${loop.loopNumber}`);
    }

    return encodeCel(cel, mirrorSourceLoopNumber);
  });
  const headerLength = 1 + encodedCels.length * 2;
  return Buffer.concat([
    Buffer.from([loop.cels.length]),
    buildHeaderForOptionalBuffers(encodedCels, headerLength),
    ...encodedCels,
  ]);
}

export function buildView(view: AGIView): Buffer {
  const mirrorSourceLoopNumbers = new Set<number>();
  const mirrorSourceLoopNumbersByTargetLoopNumber = new Map<number, number>();
  view.loops.forEach((loop, loopNumber) => {
    if (loop.cels.some((cel) => cel.mirrored)) {
      let mirrorSourceLoopNumber: number | undefined = undefined;
      loop.cels.forEach((cel) => {
        if (mirrorSourceLoopNumber != null) {
          if (cel.mirroredFromLoopNumber !== mirrorSourceLoopNumber) {
            throw new Error(`Cels in loop ${loopNumber} target different loops for mirorring!`);
          }
        } else {
          mirrorSourceLoopNumber = cel.mirroredFromLoopNumber;
        }
      });

      if (mirrorSourceLoopNumber == null) {
        throw new Error(
          `Cels in loop ${loopNumber} are marked as mirrored but have no mirroredFromLoopNumber!`,
        );
      }

      mirrorSourceLoopNumbers.add(mirrorSourceLoopNumber);
      mirrorSourceLoopNumbersByTargetLoopNumber.set(loopNumber, mirrorSourceLoopNumber);
    }
  });

  const encodedLoops = view.loops.map((loop, loopNumber) => {
    if (mirrorSourceLoopNumbersByTargetLoopNumber.has(loopNumber)) {
      // we'll fix these up in another pass
      return Buffer.from([]);
    }

    const mirrorSourceLoopNumber = mirrorSourceLoopNumbers.has(loopNumber) ? loopNumber : undefined;
    return encodeLoop(loop, mirrorSourceLoopNumber);
  });

  mirrorSourceLoopNumbersByTargetLoopNumber.forEach((mirrorSourceLoopNumber, targetLoopNumber) => {
    encodedLoops[targetLoopNumber] = encodedLoops[mirrorSourceLoopNumber];
  });

  const encodedDescription = view.description
    ? Buffer.concat([Buffer.from(view.description, 'ascii'), Buffer.from([0])])
    : undefined;
  const headerLength = 5 + encodedLoops.length * 2;

  return Buffer.concat([
    Buffer.from([
      // first 2 bytes purpose unknown
      0x01,
      0x01,
      view.loops.length,
    ]),
    buildHeaderForOptionalBuffers([encodedDescription, ...encodedLoops], headerLength),
    concatOptionalBuffers([encodedDescription, ...encodedLoops]),
  ]);
}
