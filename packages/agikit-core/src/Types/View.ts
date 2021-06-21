export type ViewCelCommon = {
  celNumber: number;
  width: number;
  height: number;
  transparentColor: number;
};

export type NonMirroredViewCel = ViewCelCommon & {
  mirrored: false;
  mirroredFromLoopNumber: undefined;
  buffer: Uint8Array;
};

export type MirroredViewCel = ViewCelCommon & {
  mirrored: true;
  mirroredFromLoopNumber: number;
};

export type ViewCel = MirroredViewCel | NonMirroredViewCel;

export type ViewLoop = {
  loopNumber: number;
  cels: ViewCel[];
};

export type AGIView = {
  loops: ViewLoop[];
  description: string | undefined;
};
