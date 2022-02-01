import { ViewCel } from '@agikit/core';
import { DrawingCanvas } from './DrawingCanvas';

export const ViewCelCanvas = ({
  cel,
  zoom,
  ...canvasProps
}: {
  cel: ViewCel;
  buffer: Uint8Array;
  zoom: number;
} & Omit<
  Parameters<typeof DrawingCanvas>[0],
  'sourceWidth' | 'sourceHeight' | 'canvasWidth' | 'canvasHeight'
>) => {
  return (
    <div
      className="transparent-backdrop"
      style={{ width: `${cel.width * zoom * 2}px`, height: `${cel.height * zoom}px` }}
    >
      <DrawingCanvas
        sourceWidth={cel.width}
        sourceHeight={cel.height}
        canvasWidth={cel.width * zoom * 2}
        canvasHeight={cel.height * zoom}
        {...canvasProps}
      />
    </div>
  );
};
