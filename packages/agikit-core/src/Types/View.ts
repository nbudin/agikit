export type ViewCelCommon = {
  width: number;
  height: number;
  transparentColor: number;
  buffer: Uint8Array;
};

export type NonMirroredViewCel = ViewCelCommon & {
  mirrored: false;
  mirroredFromLoop: undefined;
};

export type MirroredViewCel = ViewCelCommon & {
  mirrored: true;
  mirroredFromLoop: ViewLoop;
};

export type ViewCel = MirroredViewCel | NonMirroredViewCel;

export type ViewLoop = {
  cels: ViewCel[];
};

export type AGIView = {
  loops: ViewLoop[];
  description: string | undefined;
};
