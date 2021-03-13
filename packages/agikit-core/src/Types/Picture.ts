export type PictureCoordinate = {
  x: number;
  y: number;
};

export type PictureCornerStep = {
  axis: 'x' | 'y';
  position: number;
};

export type PictureRelativeLineDisplacement = {
  xDisplacement: number;
  yDisplacement: number;
};

export type PicturePenSettings = {
  splatter: boolean;
  shape: 'rectangle' | 'circle';
  size: number;
};

export type PicturePenPlotPoint = {
  position: PictureCoordinate;
  texture: number | undefined;
};

export type SetPictureColorPictureCommand = {
  type: 'SetPictureColor';
  opcode: 0xf0;
  colorNumber: number;
};

export type DisablePictureDrawPictureCommand = {
  type: 'DisablePictureDraw';
  opcode: 0xf1;
};

export type SetPriorityColorPictureCommand = {
  type: 'SetPriorityColor';
  opcode: 0xf2;
  colorNumber: number;
};

export type DisablePriorityDrawPictureCommand = {
  type: 'DisablePriorityDraw';
  opcode: 0xf3;
};

export type DrawYCornerPictureCommand = {
  type: 'DrawYCorner';
  opcode: 0xf4;
  startPosition: PictureCoordinate;
  steps: PictureCornerStep[];
};

export type DrawXCornerPictureCommand = {
  type: 'DrawXCorner';
  opcode: 0xf5;
  startPosition: PictureCoordinate;
  steps: PictureCornerStep[];
};

export type AbsoluteLinePictureCommand = {
  type: 'AbsoluteLine';
  opcode: 0xf6;
  points: PictureCoordinate[];
};

export type RelativeLinePictureCommand = {
  type: 'RelativeLine';
  opcode: 0xf7;
  startPosition: PictureCoordinate;
  relativePoints: PictureCoordinate[];
};

export type FillPictureCommand = {
  type: 'Fill';
  opcode: 0xf8;
  startPositions: PictureCoordinate[];
};

export type ChangePenPictureCommand = {
  type: 'ChangePen';
  opcode: 0xf9;
  settings: PicturePenSettings;
};

export type PlotWithPenPictureCommand = {
  type: 'PlotWithPen';
  opcode: 0xfa;
  points: PicturePenPlotPoint[];
};

export type PictureCommand =
  | SetPictureColorPictureCommand
  | DisablePictureDrawPictureCommand
  | SetPriorityColorPictureCommand
  | DisablePriorityDrawPictureCommand
  | DrawYCornerPictureCommand
  | DrawXCornerPictureCommand
  | AbsoluteLinePictureCommand
  | RelativeLinePictureCommand
  | FillPictureCommand
  | ChangePenPictureCommand
  | PlotWithPenPictureCommand;

export type PictureResource = {
  commands: PictureCommand[];
};
