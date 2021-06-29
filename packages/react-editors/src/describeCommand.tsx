import { ReactNode } from 'react';
import { EGAPalette } from '@agikit/core/dist/ColorPalettes';
import { PictureCommand, PictureCoordinate } from '@agikit/core/dist/Types/Picture';

export function describePoint(point: PictureCoordinate) {
  return `(${point.x}, ${point.y})`;
}

export function describeCommand(command: PictureCommand): ReactNode {
  if (command.type === 'AbsoluteLine') {
    return `Absolute line with points: ${command.points.map(describePoint).join(', ')}`;
  }

  if (command.type === 'RelativeLine') {
    return `Relative line from ${describePoint(
      command.startPosition,
    )} with points: ${command.relativePoints.map(describePoint).join(', ')}`;
  }

  if (command.type === 'DrawXCorner' || command.type === 'DrawYCorner') {
    return `${command.type} from ${describePoint(
      command.startPosition,
    )} with steps: ${command.steps.map((step) => `${step.axis} to ${step.position}`).join(', ')}`;
  }

  if (command.type === 'PlotWithPen') {
    return `Plot with pen at points: ${command.points
      .map(
        (plotPoint) =>
          `${describePoint(plotPoint.position)}${
            plotPoint.texture == null ? '' : `[${plotPoint.texture}]`
          }`,
      )
      .join(', ')}`;
  }

  if (command.type === 'Fill') {
    return `Fill at points: ${command.startPositions.map(describePoint).join(', ')}`;
  }

  if (command.type === 'ChangePen') {
    return `Change pen to ${JSON.stringify(command.settings)}`;
  }

  if (command.type === 'SetPictureColor' || command.type === 'SetPriorityColor') {
    const color = EGAPalette.colors[command.colorNumber];
    if (!color) {
      return `${command.type} color #${command.colorNumber}`;
    }

    return (
      <span
        style={{
          color: `rgb(${color[0]}, ${color[1]}, ${color[2]})`,
          backgroundColor: command.colorNumber < 10 ? 'white' : 'black',
        }}
      >
        {command.type} {command.colorNumber}
      </span>
    );
  }

  return command.type;
}
