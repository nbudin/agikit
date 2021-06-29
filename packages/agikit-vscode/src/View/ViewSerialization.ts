import { NonMirroredViewCel } from '@agikit/core/dist/Types/View';
import {
  EditingMirroredLoop,
  EditingRegularLoop,
  EditingView,
} from '@agikit/react-editors/dist/EditingViewTypes';

export type SerializedNonMirroredViewCel = Omit<NonMirroredViewCel, 'buffer'> & {
  bufferBase64: string;
};

export type SerializedViewLoop =
  | EditingMirroredLoop
  | (Omit<EditingRegularLoop, 'cels'> & {
      cels: SerializedNonMirroredViewCel[];
    });

export type SerializedView = Omit<EditingView, 'loops'> & {
  loops: SerializedViewLoop[];
};

export function serializeView(view: EditingView): SerializedView {
  return {
    ...view,
    loops: view.loops.map((loop) => {
      if (loop.type === 'mirrored') {
        return { ...loop };
      }

      return {
        ...loop,
        cels: loop.cels.map<SerializedNonMirroredViewCel>((cel) => ({
          celNumber: cel.celNumber,
          width: cel.width,
          height: cel.height,
          mirrored: false,
          mirroredFromLoopNumber: undefined,
          transparentColor: cel.transparentColor,
          bufferBase64: Buffer.from(cel.buffer.buffer).toString('base64'),
        })),
      };
    }),
  };
}

export function deserializeView(view: SerializedView): EditingView {
  return {
    ...view,
    loops: view.loops.map((loop) => {
      if (loop.type === 'mirrored') {
        return loop;
      }

      return {
        ...loop,
        cels: loop.cels.map<NonMirroredViewCel>((cel) => ({
          celNumber: cel.celNumber,
          height: cel.height,
          width: cel.width,
          mirrored: false,
          mirroredFromLoopNumber: undefined,
          transparentColor: cel.transparentColor,
          buffer: Buffer.from(cel.bufferBase64, 'base64'),
        })),
      };
    }),
  };
}
