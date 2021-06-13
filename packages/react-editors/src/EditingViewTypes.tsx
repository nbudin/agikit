import {
  NonMirroredViewCel,
  ViewCel,
  ViewLoop,
  AGIView,
  MirroredViewCel,
} from 'agikit-core/dist/Types/View';
import { ViewEditorCommand } from './ViewEditorCommands';

export type EditingRegularLoop = {
  type: 'regular';
  cels: NonMirroredViewCel[];
};

export type EditingMirroredLoop = {
  type: 'mirrored';
  mirroredFromLoop: EditingRegularLoop;
  cels: MirroredViewCel[];
};

export type EditingViewLoop = EditingRegularLoop | EditingMirroredLoop;

export type EditingView = {
  loops: EditingViewLoop[];
  description: string | undefined;
  commands: ViewEditorCommand[];
};

export function flipCel(cel: ViewCel): NonMirroredViewCel {
  const flippedBuffer = new Uint8Array(cel.buffer.length);

  for (let x = 0; x < cel.width; x++) {
    for (let y = 0; y < cel.height; y++) {
      const flippedX = cel.width - 1 - x;
      const offset = y * cel.width + x;
      const flippedOffset = y * cel.width + flippedX;
      flippedBuffer[flippedOffset] = cel.buffer[offset];
    }
  }

  return {
    ...cel,
    mirrored: false,
    mirroredFromLoop: undefined,
    buffer: flippedBuffer,
  };
}

export function buildEditingRegularLoop(loop: ViewLoop): EditingRegularLoop {
  return {
    type: 'regular',
    cels: loop.cels.map((cel) => {
      if (cel.mirrored) {
        return flipCel(cel);
      }

      return cel;
    }),
  };
}

export function buildEditingView(view: AGIView): EditingView {
  const editingLoops: EditingViewLoop[] = [];
  const loopNumbers = new Map<ViewLoop, number>(
    view.loops.map((loop, loopNumber) => [loop, loopNumber]),
  );
  const mirrorTargets = new Map<number, number>();

  view.loops.forEach((loop, loopNumber) => {
    if (loop.cels.every((cel) => cel.mirrored)) {
      const mirroredFromLoop = loop.cels[0].mirroredFromLoop;
      if (mirroredFromLoop && loop.cels.every((cel) => cel.mirroredFromLoop === mirroredFromLoop)) {
        const mirrorTargetNumber = loopNumbers.get(mirroredFromLoop);
        if (mirrorTargetNumber != null) {
          mirrorTargets.set(loopNumber, mirrorTargetNumber);
          return;
        }
      }
    }

    editingLoops[loopNumber] = buildEditingRegularLoop(loop);
  });

  mirrorTargets.forEach((targetLoopNumber, mirroredLoopNumber) => {
    const targetLoop = editingLoops[targetLoopNumber];
    if (targetLoop.type === 'mirrored') {
      throw new Error('Mirrored loops cannot target other mirrored loops');
    }

    editingLoops[mirroredLoopNumber] = {
      type: 'mirrored',
      mirroredFromLoop: targetLoop,
      cels: view.loops[mirroredLoopNumber].cels as MirroredViewCel[],
    };
  });

  return { ...view, loops: editingLoops, commands: [] };
}
