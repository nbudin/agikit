import assertNever from 'assert-never';
import { flatMap } from 'lodash';
import { PicBitstreamWriter } from '../Compression/Bitstreams';
import {
  PictureCommand,
  PictureCoordinate,
  PicturePenPlotPoint,
  PicturePenSettings,
  Picture,
} from '../Types/Picture';

function encodeCoordinateList(coordinates: PictureCoordinate[]) {
  return flatMap(coordinates, (point) => [point.x, point.y]);
}

function encodeSignedDisplacementNybble(displacement: number) {
  if (displacement < -7 || displacement > 7) {
    throw new Error(`Displacement ${displacement} is out of usable range (which is -7 through 7)`);
  }

  return (displacement < 0 ? 0b1000 : 0) | Math.abs(displacement);
}

function encodeDisplacementList(displacements: PictureCoordinate[]) {
  return flatMap(
    displacements,
    (displacement) =>
      (encodeSignedDisplacementNybble(displacement.x) << 4) |
      encodeSignedDisplacementNybble(displacement.y),
  );
}

function encodePenSettings(settings: PicturePenSettings) {
  if (settings.size > 7 || settings.size < 0) {
    throw new Error(`Invalid pen size ${settings.size} (valid sizes are 0 through 7)`);
  }

  return (
    (settings.splatter ? 0b100000 : 0) |
    (settings.shape === 'rectangle' ? 0b10000 : 0) |
    settings.size
  );
}

function encodePenPlotPoints(splatterEnabled: boolean, points: PicturePenPlotPoint[]) {
  return flatMap(points, (point) => {
    if (!splatterEnabled) {
      return encodeCoordinateList([point.position]);
    }

    if (point.texture == null) {
      throw new Error('Plot point has no texture set, but splatter is enabled');
    }

    return [point.texture, ...encodeCoordinateList([point.position])];
  });
}

export function compilePictureCommand(
  command: PictureCommand,
  writer: PicBitstreamWriter,
  compressColorNumbers: boolean,
): void {
  let splatterEnabled = false;

  const writeBytes = (bytes: number[]) => {
    for (const byte of bytes) {
      writer.writeCode(byte, 8);
    }
  };

  writer.writeCode(command.opcode, 8);

  if (command.type === 'DisablePictureDraw' || command.type === 'DisablePriorityDraw') {
    // no parameters for these command types
  } else if (command.type === 'SetPictureColor' || command.type === 'SetPriorityColor') {
    writer.writeCode(command.colorNumber, compressColorNumbers ? 4 : 8);
  } else if (command.type === 'DrawXCorner' || command.type === 'DrawYCorner') {
    writeBytes(encodeCoordinateList([command.startPosition]));
    writeBytes(command.steps.map((step) => step.position));
  } else if (command.type === 'AbsoluteLine') {
    writeBytes(encodeCoordinateList(command.points));
  } else if (command.type === 'RelativeLine') {
    writeBytes(encodeCoordinateList([command.startPosition]));
    writeBytes(encodeDisplacementList(command.relativePoints));
  } else if (command.type === 'Fill') {
    writeBytes(encodeCoordinateList(command.startPositions));
  } else if (command.type === 'ChangePen') {
    splatterEnabled = command.settings.splatter;
    writeBytes([encodePenSettings(command.settings)]);
  } else if (command.type === 'PlotWithPen') {
    writeBytes(encodePenPlotPoints(splatterEnabled, command.points));
  } else {
    assertNever(command);
  }
}

export function buildPicture(pictureResource: Picture, compressColorNumbers: boolean): Buffer {
  const writer = new PicBitstreamWriter();
  for (const command of pictureResource.commands) {
    compilePictureCommand(command, writer, compressColorNumbers);
  }
  writer.writeCode(0xff, 8);
  return writer.finish();
}
