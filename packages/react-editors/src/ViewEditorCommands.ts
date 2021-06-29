import assertNever from 'assert-never';
import { MirroredViewCel, NonMirroredViewCel, ViewCel } from '@agikit/core/dist/Types/View';
import {
  EditingMirroredLoop,
  EditingRegularLoop,
  EditingView,
  EditingViewLoop,
  flipCelBuffer,
} from './EditingViewTypes';
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

type ViewEditorAddLoopCommand = ViewEditorCommandCommon & {
  type: 'AddLoop';
  newLoopNumber: number;
  mirrorTargetNumber: number | undefined;
};

type ViewEditorDeleteLoopCommand = ViewEditorCommandCommon & {
  type: 'DeleteLoop';
  loop: number;
};

type ViewEditorSwapLoopsCommand = ViewEditorCommandCommon & {
  type: 'SwapLoops';
  a: number;
  b: number;
};

type ViewEditorAddCelCommand = ViewEditorCommandCommon & {
  type: 'AddCel';
  loop: number;
  newCelNumber: number;
  width: number;
  height: number;
  transparentColor: number;
};

type ViewEditorDeleteCelCommand = ViewEditorCommandCommon & {
  type: 'DeleteCel';
  loop: number;
  cel: number;
};

type ViewEditorSwapCelsCommand = ViewEditorCommandCommon & {
  type: 'SwapCels';
  loop: number;
  a: number;
  b: number;
};

export type ViewEditorCommand =
  | ViewEditorBrushCommand
  | ViewEditorResizeCommand
  | ViewEditorChangeTransparentColorCommand
  | ViewEditorAddLoopCommand
  | ViewEditorDeleteLoopCommand
  | ViewEditorSwapLoopsCommand
  | ViewEditorAddCelCommand
  | ViewEditorDeleteCelCommand
  | ViewEditorSwapCelsCommand;

export function updateCel(
  view: EditingView,
  loopNumber: number,
  celNumber: number,
  update: (cel: NonMirroredViewCel) => NonMirroredViewCel,
): EditingView {
  return {
    ...view,
    loops: view.loops.map((loop) => {
      if (loop.loopNumber === loopNumber) {
        if (loop.type === 'mirrored') {
          throw new Error(
            `Can't run updateCel on a mirrored cel (loop ${loopNumber}, cel ${celNumber})`,
          );
        }

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

export function updateLoop(
  view: EditingView,
  loopNumber: number,
  update: (loop: EditingRegularLoop) => EditingRegularLoop,
  updateMirror: (mirrorLoop: EditingMirroredLoop) => EditingMirroredLoop,
): EditingView {
  const loopToUpdate = view.loops[loopNumber];
  if (loopToUpdate == null) {
    throw new Error(`Can't update nonexistent loop ${loopNumber}`);
  }
  if (loopToUpdate.type === 'mirrored') {
    throw new Error(`Can't add cel to mirrored loop ${loopNumber}`);
  }

  return {
    ...view,
    loops: view.loops.map((loop) => {
      if (loop.loopNumber === loopNumber) {
        return update(loopToUpdate);
      } else if (loopToUpdate.mirroredByLoopNumbers.includes(loop.loopNumber)) {
        if (loop.type !== 'mirrored') {
          throw new Error(
            `Loop ${loop.loopNumber} is supposed to be a mirror of ${loopNumber}, but is a regular loop`,
          );
        }
        return updateMirror(loop);
      }
      return loop;
    }),
  };
}

function celIsMirrored(cel: ViewCel): cel is MirroredViewCel {
  return cel.mirrored;
}

export function updateCelForLoopRenumbering<T extends ViewCel>(
  cel: T,
  oldLoopNumberToNewLoopNumber: (oldLoopNumber: number) => number,
): T {
  if (celIsMirrored(cel)) {
    return {
      ...cel,
      mirroredFromLoopNumber: oldLoopNumberToNewLoopNumber(cel.mirroredFromLoopNumber),
    };
  }
  return cel;
}

export function renumberLoops(
  view: EditingView,
  oldLoopNumberToNewLoopNumber: (oldLoopNumber: number) => number,
) {
  return view.loops.map<EditingViewLoop>((loop) => {
    if (loop.type === 'mirrored') {
      return {
        ...loop,
        loopNumber: oldLoopNumberToNewLoopNumber(loop.loopNumber),
        mirroredFromLoopNumber: oldLoopNumberToNewLoopNumber(loop.mirroredFromLoopNumber),
      };
    }

    return {
      ...loop,
      loopNumber: oldLoopNumberToNewLoopNumber(loop.loopNumber),
      mirroredByLoopNumbers: loop.mirroredByLoopNumbers.map(oldLoopNumberToNewLoopNumber),
      cels: loop.cels.map((cel) => updateCelForLoopRenumbering(cel, oldLoopNumberToNewLoopNumber)),
    };
  });
}

export function renumberCelsByIndex(newLoop: {
  cels: NonMirroredViewCel[];
  loopNumber: number;
  type: 'regular';
  mirroredByLoopNumbers: number[];
}): EditingRegularLoop {
  return {
    ...newLoop,
    cels: newLoop.cels.map((cel, index) => ({ ...cel, celNumber: index })),
  };
}

export function resizeCel(
  cel: NonMirroredViewCel,
  width: number,
  height: number,
  originX: number,
  originY: number,
) {
  const newCel = {
    ...cel,
    width: width,
    height: height,
    buffer: new Uint8Array({ length: width * height }),
  };

  for (let targetX = 0; targetX < newCel.width; targetX++) {
    for (let targetY = 0; targetY < newCel.height; targetY++) {
      const offset = targetX + targetY * width;
      const sourceX = targetX + originX;
      const sourceY = targetY + originY;
      if (sourceX < cel.width && sourceY < cel.height) {
        newCel.buffer[offset] = cel.buffer[sourceX + sourceY * cel.width]!;
      } else {
        newCel.buffer[offset] = cel.transparentColor;
      }
    }
  }

  return newCel;
}

export function addLoop(
  view: EditingView,
  newLoopNumber: number,
  mirrorTargetNumber: number | undefined,
) {
  const newLoop: EditingViewLoop =
    mirrorTargetNumber == null
      ? {
          loopNumber: newLoopNumber,
          cels: [],
          type: 'regular',
          mirroredByLoopNumbers: [],
        }
      : {
          loopNumber: newLoopNumber,
          type: 'mirrored',
          mirroredFromLoopNumber: mirrorTargetNumber,
        };

  const newLoops = renumberLoops(view, (oldLoopNumber: number) => {
    if (oldLoopNumber >= newLoopNumber) {
      return oldLoopNumber + 1;
    }
    return oldLoopNumber;
  });
  if (mirrorTargetNumber != null) {
    const mirrorTargetLoop = newLoops[mirrorTargetNumber];
    if (mirrorTargetLoop == null) {
      throw new Error(`Nonexistent mirror target loop number ${mirrorTargetNumber}`);
    }
    if (mirrorTargetLoop.type === 'mirrored') {
      throw new Error(`Can't set mirror target to a mirrored loop (${mirrorTargetNumber})`);
    }
    mirrorTargetLoop.mirroredByLoopNumbers.push(newLoopNumber);
  }
  newLoops.splice(newLoopNumber, 0, newLoop);

  return {
    ...view,
    loops: newLoops,
  };
}

export function deleteLoop(view: EditingView, loopNumberToDelete: number) {
  const loopToDelete = view.loops[loopNumberToDelete];
  if (loopToDelete == null) {
    throw new Error(`Can't delete nonexistent loop ${loopNumberToDelete}`);
  }

  let demirroredView = view;

  if (loopToDelete.type === 'regular') {
    const firstMirrorNumber = loopToDelete.mirroredByLoopNumbers[0];
    if (firstMirrorNumber != null) {
      demirroredView = {
        ...view,
        loops: view.loops.map<EditingViewLoop>((loop) => {
          if (loop.loopNumber === firstMirrorNumber) {
            return {
              type: 'regular',
              loopNumber: loop.loopNumber,
              mirroredByLoopNumbers: loopToDelete.mirroredByLoopNumbers.slice(1),
              cels: loopToDelete.cels.map<NonMirroredViewCel>((cel) => ({
                ...cel,
                mirrored: false,
                mirroredFromLoopNumber: undefined,
                buffer: flipCelBuffer(cel),
              })),
            };
          }

          if (loopToDelete.mirroredByLoopNumbers.includes(loop.loopNumber)) {
            if (loop.type !== 'mirrored') {
              throw new Error(`Loop ${loop.loopNumber} is supposed to be mirrored but isn't`);
            }

            return {
              ...loop,
              mirroredFromLoopNumber: firstMirrorNumber,
            };
          }

          return loop;
        }),
      };
    }
  }

  const newLoops = renumberLoops(demirroredView, (oldLoopNumber: number) => {
    if (oldLoopNumber > loopNumberToDelete) {
      return oldLoopNumber - 1;
    }

    return oldLoopNumber;
  });
  newLoops.splice(loopNumberToDelete, 1);

  return {
    ...demirroredView,
    loops: newLoops,
  };
}

export function swapLoops(view: EditingView, a: number, b: number): EditingView {
  const newLoops = renumberLoops(view, (oldLoopNumber: number) => {
    if (oldLoopNumber === a) {
      return b;
    }

    if (oldLoopNumber === b) {
      return a;
    }

    return oldLoopNumber;
  });

  const loopA = newLoops[a];
  const loopB = newLoops[b];
  if (!loopA) {
    throw new Error(`Can't swap nonexistent loop ${a}`);
  }
  if (!loopB) {
    throw new Error(`Can't swap nonexistent loop ${b}`);
  }

  newLoops[a] = loopB;
  newLoops[b] = loopA;
  return { ...view, loops: newLoops };
}

export function addCel(
  view: EditingView,
  loopNumber: number,
  celNumber: number,
  width: number,
  height: number,
  transparentColor: number,
): EditingView {
  return updateLoop(
    view,
    loopNumber,
    (loop) => {
      const newLoop = { ...loop, cels: [...loop.cels] };
      newLoop.cels.splice(celNumber, 0, {
        celNumber,
        mirrored: false,
        width,
        height,
        transparentColor,
        buffer: Uint8Array.from({ length: width * height }, () => transparentColor),
        mirroredFromLoopNumber: undefined,
      });
      return renumberCelsByIndex(newLoop);
    },
    (mirrorLoop) => mirrorLoop,
  );
}

export function deleteCel(view: EditingView, loopNumber: number, celNumber: number): EditingView {
  return updateLoop(
    view,
    loopNumber,
    (loop) => {
      const newLoop = { ...loop, cels: [...loop.cels] };
      newLoop.cels.splice(celNumber, 1);
      return renumberCelsByIndex(newLoop);
    },
    (mirrorLoop) => mirrorLoop,
  );
}

export function swapCels(view: EditingView, loopNumber: number, a: number, b: number): EditingView {
  return updateLoop(
    view,
    loopNumber,
    (loop) => {
      const newCels = [...loop.cels];
      const celA = newCels[a];
      const celB = newCels[b];
      if (!celA) {
        throw new Error(`Can't swap nonexistent cel ${a}`);
      }
      if (!celB) {
        throw new Error(`Can't swap nonexistent cel ${b}`);
      }
      newCels[a] = celB;
      newCels[b] = celA;
      return renumberCelsByIndex({ ...loop, cels: newCels });
    },
    (mirrorLoop) => mirrorLoop,
  );
}

export function applyViewEditorCommand(view: EditingView, command: ViewEditorCommand): EditingView {
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

      return resizeCel(cel, command.width, command.height, command.originX, command.originY);
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

  if (command.type === 'AddLoop') {
    return addLoop(view, command.newLoopNumber, command.mirrorTargetNumber);
  }

  if (command.type === 'DeleteLoop') {
    return deleteLoop(view, command.loop);
  }

  if (command.type === 'SwapLoops') {
    return swapLoops(view, command.a, command.b);
  }

  if (command.type === 'AddCel') {
    return addCel(
      view,
      command.loop,
      command.newCelNumber,
      command.width,
      command.height,
      command.transparentColor,
    );
  }

  if (command.type === 'DeleteCel') {
    return deleteCel(view, command.loop, command.cel);
  }

  if (command.type === 'SwapCels') {
    return swapCels(view, command.loop, command.a, command.b);
  }

  assertNever(command);
}

export function applyViewEditorCommands(
  view: EditingView,
  commands: ViewEditorCommand[],
): EditingView {
  return commands.reduce(
    (workingView, command) => applyViewEditorCommand(workingView, command),
    view,
  );
}
