import {
  AGIView,
  MirroredViewCel,
  NonMirroredViewCel,
  ViewLoop,
} from '../../../agikit-core/dist/Types/View';

export type SerializedNonMirroredViewCel = Omit<NonMirroredViewCel, 'buffer'> & {
  buffer: number[];
};

export type SerializedViewCel = MirroredViewCel | SerializedNonMirroredViewCel;

export type SerializedViewLoop = Omit<ViewLoop, 'cels'> & {
  cels: SerializedViewCel[];
};

export type SerializedView = Omit<AGIView, 'loops'> & {
  loops: SerializedViewLoop[];
};

export function serializeView(view: AGIView): SerializedView {
  return {
    ...view,
    loops: view.loops.map((loop) => ({
      ...loop,
      cels: loop.cels.map((cel) => {
        if (cel.mirrored) {
          return cel;
        }

        return {
          ...cel,
          buffer: [...cel.buffer],
        };
      }),
    })),
  };
}

export function deserializeView(view: SerializedView): AGIView {
  return {
    ...view,
    loops: view.loops.map((loop) => ({
      ...loop,
      cels: loop.cels.map((cel) => {
        if (cel.mirrored) {
          return cel;
        }

        return {
          ...cel,
          buffer: Uint8Array.from(cel.buffer),
        };
      }),
    })),
  };
}
