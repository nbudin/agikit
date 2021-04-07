import { PIC_HEIGHT, PIC_WIDTH } from './constants';
import { DrawingCanvas } from './DrawingCanvas';

export function PicCanvas(
  props: Omit<Parameters<typeof DrawingCanvas>[0], 'sourceWidth' | 'sourceHeight'>,
) {
  return <DrawingCanvas {...props} sourceWidth={PIC_WIDTH} sourceHeight={PIC_HEIGHT} />;
}
