import { BitstreamReader } from '../../Compression/Bitstreams';
import {
  Picture,
  PictureCommand,
  PictureCommandOpcodes,
  PictureCoordinate,
  PictureCornerStep,
  PicturePenPlotPoint,
} from '../../Types/Picture';

function decodeSignedDisplacementNybble(nybble: number) {
  const sign = (nybble & 0b1000) === 0b1000 ? -1 : 1;
  return (nybble & 0b111) * sign;
}

export function readPictureResource(data: Buffer, compressColorNumbers: boolean): Picture {
  const reader = new BitstreamReader(data);
  const commands: PictureCommand[] = [];
  let splatterEnabled = false;

  const consumeCoordinates = () => {
    const x = reader.readCode(8);
    const y = reader.readCode(8);
    return { x, y };
  };

  const consumeCornerSteps = (startAxis: 'x' | 'y') => {
    const steps: PictureCornerStep[] = [];
    let axis = startAxis;
    let currentByte: number;
    do {
      currentByte = reader.peekCode(8);
      if (currentByte < 0xf0) {
        reader.seekBits(8);
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
      currentByte = reader.peekCode(8);
      if (currentByte < 0xf0) {
        reader.seekBits(8);
        points.push({ x: currentByte, y: reader.readCode(8) });
      }
    } while (currentByte < 0xf0);
    return points;
  };

  const consumeRelativeCoordinateArray = () => {
    const points: PictureCoordinate[] = [];
    let currentByte: number;
    do {
      currentByte = reader.peekCode(8);
      if (currentByte < 0xf0) {
        reader.seekBits(8);
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
      currentByte = reader.peekCode(8);
      if (currentByte < 0xf0) {
        reader.seekBits(8);

        if (splatterEnabled) {
          const texture = currentByte;
          const position = consumeCoordinates();
          plotPoints.push({ texture, position });
        } else {
          const y = reader.readCode(8);
          plotPoints.push({ texture: undefined, position: { x: currentByte, y } });
        }
      }
    } while (currentByte < 0xf0);
    return plotPoints;
  };

  while (!reader.done()) {
    const opcode = reader.readCode(8);

    if (opcode === PictureCommandOpcodes.SetPictureColor) {
      commands.push({
        opcode,
        type: 'SetPictureColor',
        colorNumber: reader.readCode(compressColorNumbers ? 4 : 8),
      });
    } else if (opcode === PictureCommandOpcodes.DisablePictureDraw) {
      commands.push({
        opcode,
        type: 'DisablePictureDraw',
      });
    } else if (opcode === PictureCommandOpcodes.SetPriorityColor) {
      commands.push({
        opcode,
        type: 'SetPriorityColor',
        colorNumber: reader.readCode(compressColorNumbers ? 4 : 8),
      });
    } else if (opcode === PictureCommandOpcodes.DisablePriorityDraw) {
      commands.push({
        opcode,
        type: 'DisablePriorityDraw',
      });
    } else if (opcode === PictureCommandOpcodes.DrawYCorner) {
      const startPosition = consumeCoordinates();
      const steps = consumeCornerSteps('y');
      commands.push({
        opcode,
        type: 'DrawYCorner',
        startPosition,
        steps,
      });
    } else if (opcode === PictureCommandOpcodes.DrawXCorner) {
      const startPosition = consumeCoordinates();
      const steps = consumeCornerSteps('x');
      commands.push({
        opcode,
        type: 'DrawXCorner',
        startPosition,
        steps,
      });
    } else if (opcode === PictureCommandOpcodes.AbsoluteLine) {
      const points = consumeAbsoluteCoordinateArray();
      commands.push({
        opcode,
        type: 'AbsoluteLine',
        points,
      });
    } else if (opcode === PictureCommandOpcodes.RelativeLine) {
      const startPosition = consumeCoordinates();
      const relativePoints = consumeRelativeCoordinateArray();
      commands.push({
        opcode,
        type: 'RelativeLine',
        startPosition,
        relativePoints,
      });
    } else if (opcode === PictureCommandOpcodes.Fill) {
      const startPositions = consumeAbsoluteCoordinateArray();
      commands.push({
        opcode,
        type: 'Fill',
        startPositions,
      });
    } else if (opcode === PictureCommandOpcodes.ChangePen) {
      const settingsByte = reader.readCode(8);
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
    } else if (opcode === PictureCommandOpcodes.PlotWithPen) {
      const points = consumePenPlotPoints();
      commands.push({
        opcode,
        type: 'PlotWithPen',
        points,
      });
    } else if (opcode === 0xff) {
      // skip it, it's a no-op
    } else {
      throw new Error(
        `Unknown picture opcode ${opcode.toString(16)} at offset ${reader.byteOffset}`,
      );
    }
  }

  return { commands };
}
