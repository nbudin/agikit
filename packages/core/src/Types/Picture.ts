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

export const PictureCommandOpcodes = {
  SetPictureColor: 0xf0,
  DisablePictureDraw: 0xf1,
  SetPriorityColor: 0xf2,
  DisablePriorityDraw: 0xf3,
  DrawYCorner: 0xf4,
  DrawXCorner: 0xf5,
  AbsoluteLine: 0xf6,
  RelativeLine: 0xf7,
  Fill: 0xf8,
  ChangePen: 0xf9,
  PlotWithPen: 0xfa,
} as const;

export type SetPictureColorPictureCommand = {
  type: 'SetPictureColor';
  opcode: typeof PictureCommandOpcodes['SetPictureColor'];
  colorNumber: number;
};

export type DisablePictureDrawPictureCommand = {
  type: 'DisablePictureDraw';
  opcode: typeof PictureCommandOpcodes['DisablePictureDraw'];
};

export type SetPriorityColorPictureCommand = {
  type: 'SetPriorityColor';
  opcode: typeof PictureCommandOpcodes['SetPriorityColor'];
  colorNumber: number;
};

export type DisablePriorityDrawPictureCommand = {
  type: 'DisablePriorityDraw';
  opcode: typeof PictureCommandOpcodes['DisablePriorityDraw'];
};

export type DrawYCornerPictureCommand = {
  type: 'DrawYCorner';
  opcode: typeof PictureCommandOpcodes['DrawYCorner'];
  startPosition: PictureCoordinate;
  steps: PictureCornerStep[];
};

export type DrawXCornerPictureCommand = {
  type: 'DrawXCorner';
  opcode: typeof PictureCommandOpcodes['DrawXCorner'];
  startPosition: PictureCoordinate;
  steps: PictureCornerStep[];
};

export type AbsoluteLinePictureCommand = {
  type: 'AbsoluteLine';
  opcode: typeof PictureCommandOpcodes['AbsoluteLine'];
  points: PictureCoordinate[];
};

export type RelativeLinePictureCommand = {
  type: 'RelativeLine';
  opcode: typeof PictureCommandOpcodes['RelativeLine'];
  startPosition: PictureCoordinate;
  relativePoints: PictureCoordinate[];
};

export type FillPictureCommand = {
  type: 'Fill';
  opcode: typeof PictureCommandOpcodes['Fill'];
  startPositions: PictureCoordinate[];
};

export type ChangePenPictureCommand = {
  type: 'ChangePen';
  opcode: typeof PictureCommandOpcodes['ChangePen'];
  settings: PicturePenSettings;
};

export type PlotWithPenPictureCommand = {
  type: 'PlotWithPen';
  opcode: typeof PictureCommandOpcodes['PlotWithPen'];
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

export type Picture = {
  commands: PictureCommand[];
};
