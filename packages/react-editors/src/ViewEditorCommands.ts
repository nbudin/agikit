import assertNever from 'assert-never';
import { AGIView, ViewCel, ViewLoop } from '../../agikit-core/dist/Types/View';
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

export type ViewEditorCommand = ViewEditorBrushCommand;

export function updateCel(
  view: AGIView,
  loopNumber: number,
  celNumber: number,
  update: (cel: ViewCel) => ViewCel,
): AGIView {
  return {
    ...view,
    loops: view.loops.map((loop) => {
      if (loop.loopNumber === loopNumber) {
        return {
          ...loop,
          cels: view.loops[loopNumber].cels.map((cel) => {
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

export function applyViewEditorCommand(view: AGIView, command: ViewEditorCommand): AGIView {
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

  assertNever(command.type);
}

export function applyViewEditorCommands(view: AGIView, commands: ViewEditorCommand[]): AGIView {
  return commands.reduce(
    (workingCel, command) => applyViewEditorCommand(workingCel, command),
    view,
  );
}
