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

export type ViewEditorCommand = ViewEditorBrushCommand;

export function applyViewEditorCommand(view: AGIView, command: ViewEditorCommand): AGIView {
  if (command.type === 'Brush') {
    return {
      ...view,
      loops: view.loops.map((loop, loopNumber) => {
        if (loopNumber === command.loop) {
          return {
            ...loop,
            cels: loop.cels.map((cel, celNumber) => {
              if (celNumber === command.cel) {
                return { ...cel, buffer: applyBrushStroke(cel.buffer, cel, command.brushStroke) };
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

  assertNever(command.type);
}

export function applyViewEditorCommands(view: AGIView, commands: ViewEditorCommand[]): AGIView {
  return commands.reduce(
    (workingCel, command) => applyViewEditorCommand(workingCel, command),
    view,
  );
}
