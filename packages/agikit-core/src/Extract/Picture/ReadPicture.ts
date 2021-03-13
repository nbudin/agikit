import {
  PictureCommand,
  PictureCoordinate,
  PictureCornerStep,
  PicturePenPlotPoint,
  PictureResource,
} from '../../Types/Picture';

function decodeSignedDisplacementNybble(nybble: number) {
  const sign = (nybble & 0b1000) === 0b1000 ? -1 : 1;
  return (nybble & 0b111) * sign;
}

export function readPictureResource(data: Buffer): PictureResource {
  const commands: PictureCommand[] = [];
  let offset = 0;
  let splatterEnabled = false;

  const consumeUInt8 = () => {
    const value = data.readUInt8(offset);
    offset += 1;
    return value;
  };

  const consumeCoordinates = () => {
    const x = consumeUInt8();
    const y = consumeUInt8();
    return { x, y };
  };

  const consumeCornerSteps = (startAxis: 'x' | 'y') => {
    const steps: PictureCornerStep[] = [];
    let axis = startAxis;
    let currentByte: number;
    do {
      currentByte = data.readUInt8(offset);
      if (currentByte < 0xf0) {
        offset += 1;
        steps.push({
          axis,
          position: currentByte,
        });
        axis = axis === 'x' ? 'y' : 'x';
      }
    } while (currentByte < 0xf0);
    return steps;
  };

  const consumeAbsoluteCoordinateArray = () => {
    const points: PictureCoordinate[] = [];
    let currentByte: number;
    do {
      currentByte = data.readUInt8(offset);
      if (currentByte < 0xf0) {
        offset += 1;
        points.push({ x: currentByte, y: consumeUInt8() });
      }
    } while (currentByte < 0xf0);
    return points;
  };

  const consumeRelativeCoordinateArray = () => {
    const points: PictureCoordinate[] = [];
    let currentByte: number;
    do {
      currentByte = data.readUInt8(offset);
      if (currentByte < 0xf0) {
        offset += 1;
        const xNybble = (currentByte & 0xf0) >> 4;
        const yNybble = currentByte & 0x0f;
        points.push({
          x: decodeSignedDisplacementNybble(xNybble),
          y: decodeSignedDisplacementNybble(yNybble),
        });
      }
    } while (currentByte < 0xf0);
    return points;
  };

  const consumePenPlotPoints = () => {
    const plotPoints: PicturePenPlotPoint[] = [];
    let currentByte: number;
    do {
      currentByte = data.readUInt8(offset);
      if (currentByte < 0xf0) {
        offset += 1;
        let texture: number | undefined;
        if (splatterEnabled) {
          texture = consumeUInt8();
        }
        const position = consumeCoordinates();
        plotPoints.push({ texture, position });
      }
    } while (currentByte < 0xf0);
    return plotPoints;
  };

  while (offset < data.byteLength) {
    const opcode = consumeUInt8();

    if (opcode === 0xf0) {
      commands.push({
        opcode,
        type: 'SetPictureColor',
        colorNumber: consumeUInt8(),
      });
    } else if (opcode === 0xf1) {
      commands.push({
        opcode,
        type: 'DisablePictureDraw',
      });
    } else if (opcode === 0xf2) {
      commands.push({
        opcode,
        type: 'SetPriorityColor',
        colorNumber: consumeUInt8(),
      });
    } else if (opcode === 0xf3) {
      commands.push({
        opcode,
        type: 'DisablePriorityDraw',
      });
    } else if (opcode === 0xf4) {
      const startPosition = consumeCoordinates();
      const steps = consumeCornerSteps('y');
      commands.push({
        opcode,
        type: 'DrawYCorner',
        startPosition,
        steps,
      });
    } else if (opcode === 0xf5) {
      const startPosition = consumeCoordinates();
      const steps = consumeCornerSteps('x');
      commands.push({
        opcode,
        type: 'DrawXCorner',
        startPosition,
        steps,
      });
    } else if (opcode === 0xf6) {
      const points = consumeAbsoluteCoordinateArray();
      commands.push({
        opcode,
        type: 'AbsoluteLine',
        points,
      });
    } else if (opcode === 0xf7) {
      const startPosition = consumeCoordinates();
      const relativePoints = consumeRelativeCoordinateArray();
      commands.push({
        opcode,
        type: 'RelativeLine',
        startPosition,
        relativePoints,
      });
    } else if (opcode === 0xf8) {
      const startPositions = consumeAbsoluteCoordinateArray();
      commands.push({
        opcode,
        type: 'Fill',
        startPositions,
      });
    } else if (opcode === 0xf9) {
      const settingsByte = consumeUInt8();
      splatterEnabled = (0b100000 & settingsByte) === 0b100000;
      commands.push({
        opcode,
        type: 'ChangePen',
        settings: {
          splatter: splatterEnabled,
          shape: (0b10000 & settingsByte) === 0b10000 ? 'rectangle' : 'circle',
          size: 0b111 & settingsByte,
        },
      });
    } else if (opcode === 0xfa) {
      const points = consumePenPlotPoints();
      commands.push({
        opcode,
        type: 'PlotWithPen',
        points,
      });
    } else if (opcode === 0xff) {
      // skip it, it's a no-op
    } else {
      throw new Error(`Unknown picture opcode ${opcode.toString(16)} at offset ${offset}`);
    }
  }

  return { commands };
}
