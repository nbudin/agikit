import assertNever from 'assert-never';
import { ViewCel } from '../../agikit-core/dist/Types/View';
import { applyBrushStroke, BrushStroke } from './ViewEditorBrushStrokes';

type ViewEditorCommandCommon = {
  uuid: string;
  loop: number;
  cel: number;
};

type ViewEditorBrushCommand = ViewEditorCommandCommon & {
  type: 'Brush';
  brushStroke: BrushStroke;
};

export type ViewEditorCommand = ViewEditorBrushCommand;

export function applyViewEditorCommand(cel: ViewCel, command: ViewEditorCommand): ViewCel {
  if (command.type === 'Brush') {
    return { ...cel, buffer: applyBrushStroke(cel.buffer, cel, command.brushStroke) };
  }

  assertNever(command.type);
}

export function applyViewEditorCommands(cel: ViewCel, commands: ViewEditorCommand[]): ViewCel {
  return commands.reduce((workingCel, command) => applyViewEditorCommand(workingCel, command), cel);
}
