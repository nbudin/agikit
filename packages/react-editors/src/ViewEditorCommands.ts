import assertNever from 'assert-never';
import { AGIView, ViewCel } from '../../agikit-core/dist/Types/View';
import { applyBrushStroke, BrushStroke } from './ViewEditorBrushStrokes';

type ViewEditorCommandCommon = {
  uuid: string;
};

type ViewEditorBrushCommand = ViewEditorCommandCommon & {
  type: 'Brush';
  brushStroke: BrushStroke;
  loop: number;
  cel: number;
};

type ViewEditorResizeCommand = ViewEditorCommandCommon & {
  type: 'Resize';
  loop: number;
  cel: number;
  width: number;
  height: number;
  originX: number;
  originY: number;
};

type ViewEditorChangeTransparentColorCommand = ViewEditorCommandCommon & {
  type: 'ChangeTransparentColor';
  loop: number;
  cel: number;
  transparentColor: number;
};

export type ViewEditorCommand =
  | ViewEditorBrushCommand
  | ViewEditorResizeCommand
  | ViewEditorChangeTransparentColorCommand;

export function updateCel<T extends AGIView>(
  view: T,
  loopNumber: number,
  celNumber: number,
  update: (cel: ViewCel) => ViewCel,
): T {
  return {
    ...view,
    loops: view.loops.map((loop) => {
      if (loop.loopNumber === loopNumber) {
        return {
          ...loop,
          cels: loop.cels.map((cel) => {
            if (cel.celNumber === celNumber) {
              return update(cel);
            } else {
              return cel;
            }
          }),
        };
      } else {
        return loop;
      }
    }),
  };
}

export function applyViewEditorCommand<T extends AGIView>(view: T, command: ViewEditorCommand): T {
  if (command.type === 'Brush') {
    return updateCel(view, command.loop, command.cel, (cel) => {
      if (cel.mirrored) {
        throw new Error(
          "Can't apply brush command on a mirrored cel; brush commands must be done on source",
        );
      }

      return {
        ...cel,
        buffer: applyBrushStroke(cel.buffer, cel, command.brushStroke),
      };
    });
  }

  if (command.type === 'Resize') {
    return updateCel(view, command.loop, command.cel, (cel) => {
      if (cel.mirrored) {
        throw new Error(
          "Can't apply resize command on a mirrored cel; resize commands must be done on source",
        );
      }

      const newCel = {
        ...cel,
        width: command.width,
        height: command.height,
        buffer: new Uint8Array({ length: command.width * command.height }),
      };

      for (let targetX = 0; targetX < newCel.width; targetX++) {
        for (let targetY = 0; targetY < newCel.height; targetY++) {
          const offset = targetX + targetY * command.width;
          const sourceX = targetX + command.originX;
          const sourceY = targetY + command.originY;
          if (sourceX < cel.width && sourceY < cel.height) {
            newCel.buffer[offset] = cel.buffer[sourceX + sourceY * cel.width]!;
          } else {
            newCel.buffer[offset] = cel.transparentColor;
          }
        }
      }

      return newCel;
    });
  }

  if (command.type === 'ChangeTransparentColor') {
    return updateCel(view, command.loop, command.cel, (cel) => {
      if (cel.mirrored) {
        throw new Error(
          "Can't apply change transparent color command on a mirrored cel; change transparent color commands must be done on source",
        );
      }

      return { ...cel, transparentColor: command.transparentColor };
    });
  }

  assertNever(command);
}

export function applyViewEditorCommands<T extends AGIView>(
  view: T,
  commands: ViewEditorCommand[],
): T {
  return commands.reduce(
    (workingCel, command) => applyViewEditorCommand(workingCel, command),
    view,
  );
}
