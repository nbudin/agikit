import assertNever from 'assert-never';
import { flatMap } from 'lodash';
import {
  PictureCommand,
  PictureCoordinate,
  PicturePenPlotPoint,
  PicturePenSettings,
  PictureResource,
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
  if (settings.size > 7 || settings.size < 1) {
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

export function compilePictureCommand(command: PictureCommand): Buffer {
  let splatterEnabled = false;

  if (command.type === 'DisablePictureDraw' || command.type === 'DisablePriorityDraw') {
    return Buffer.from([command.opcode]);
  }

  if (command.type === 'SetPictureColor' || command.type === 'SetPriorityColor') {
    return Buffer.from([command.opcode, command.colorNumber]);
  }

  if (command.type === 'DrawXCorner' || command.type === 'DrawYCorner') {
    return Buffer.from([
      command.opcode,
      ...encodeCoordinateList([command.startPosition]),
      ...command.steps.map((step) => step.position),
    ]);
  }

  if (command.type === 'AbsoluteLine') {
    return Buffer.from([command.opcode, ...encodeCoordinateList(command.points)]);
  }

  if (command.type === 'RelativeLine') {
    return Buffer.from([
      command.opcode,
      ...encodeCoordinateList([command.startPosition]),
      ...encodeDisplacementList(command.relativePoints),
    ]);
  }

  if (command.type === 'Fill') {
    return Buffer.from([command.opcode, ...encodeCoordinateList(command.startPositions)]);
  }

  if (command.type === 'ChangePen') {
    splatterEnabled = command.settings.splatter;
    return Buffer.from([command.opcode, encodePenSettings(command.settings)]);
  }

  if (command.type === 'PlotWithPen') {
    return Buffer.from([command.opcode, ...encodePenPlotPoints(splatterEnabled, command.points)]);
  }

  assertNever(command);
}

export function buildPicture(pictureResource: PictureResource): Buffer {
  const compiledCommands = pictureResource.commands.map(compilePictureCommand);
  return Buffer.concat([...compiledCommands, Buffer.from([0xff])]);
}
