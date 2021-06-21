import { NonMirroredViewCel, ViewCel, AGIView, MirroredViewCel } from 'agikit-core/dist/Types/View';
import { ViewEditorCommand } from './ViewEditorCommands';

export type EditingRegularLoop = {
  loopNumber: number;
  type: 'regular';
  cels: NonMirroredViewCel[];
  mirroredByLoopNumbers: number[];
};

export type EditingMirroredLoop = {
  loopNumber: number;
  type: 'mirrored';
  mirroredFromLoopNumber: number;
  cels: MirroredViewCel[];
};

export type EditingViewLoop = EditingRegularLoop | EditingMirroredLoop;

export type EditingView = {
  loops: EditingViewLoop[];
  description: string | undefined;
  commands: ViewEditorCommand[];
};

export function flipCelBuffer(cel: NonMirroredViewCel): Uint8Array {
  const flippedBuffer = new Uint8Array(cel.buffer.length);

  for (let x = 0; x < cel.width; x++) {
    for (let y = 0; y < cel.height; y++) {
      const flippedX = cel.width - 1 - x;
      const offset = y * cel.width + x;
      const flippedOffset = y * cel.width + flippedX;
      flippedBuffer[flippedOffset] = cel.buffer[offset];
    }
  }

  return flippedBuffer;
}

export function buildEditingLoop(view: AGIView, loopNumber: number): EditingViewLoop {
  const loop = view.loops[loopNumber];
  let mirrorTarget: number | undefined;

  if (loop.cels.every((cel) => cel.mirrored)) {
    const mirroredFromLoopNumber = loop.cels[0].mirroredFromLoopNumber;
    if (
      mirroredFromLoopNumber != null &&
      loop.cels.every((cel) => cel.mirroredFromLoopNumber === mirroredFromLoopNumber)
    ) {
      mirrorTarget = mirroredFromLoopNumber;
    }
  }

  if (mirrorTarget != null) {
    return {
      ...loop,
      type: 'mirrored',
      mirroredFromLoopNumber: mirrorTarget,
      cels: loop.cels.map((cel) => {
        const sourceCel = view.loops[mirrorTarget as number].cels[cel.celNumber];
        if (sourceCel.mirrored) {
          throw new Error('Mirrored cel points at another mirrored cel');
        }

        return {
          ...cel,
          mirrored: true,
          mirroredFromLoopNumber: mirrorTarget as number,
          buffer: flipCelBuffer(sourceCel),
        };
      }),
    };
  } else {
    return {
      ...loop,
      type: 'regular',
      mirroredByLoopNumbers: [],
      cels: loop.cels.map((cel) => {
        if (cel.mirrored) {
          throw new Error('Mirrored cel found in a regular loop');
        }

        return {
          ...cel,
          mirrored: false,
          mirroredFromLoopNumber: undefined,
        };
      }),
    };
  }
}

export function buildEditingView(view: AGIView): EditingView {
  const editingLoops: EditingViewLoop[] = [];
  const mirrorTargets = new Map<number, number>();

  view.loops.forEach((loop, loopNumber) => {
    const editingLoop = buildEditingLoop(view, loopNumber);
    if (editingLoop.type === 'mirrored') {
      mirrorTargets.set(editingLoop.loopNumber, editingLoop.mirroredFromLoopNumber);
    }
    editingLoops[loopNumber] = editingLoop;
  });

  mirrorTargets.forEach((targetLoopNumber, mirroredLoopNumber) => {
    const targetLoop = editingLoops[targetLoopNumber];
    if (targetLoop.type === 'mirrored') {
      throw new Error('Mirrored loops cannot target other mirrored loops');
    }

    targetLoop.mirroredByLoopNumbers.push(mirroredLoopNumber);
  });

  return { ...view, loops: editingLoops, commands: [] };
}
