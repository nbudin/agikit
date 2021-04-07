import { clamp } from 'lodash';

export function calculateClampedPosition(
  clientPosition: number,
  offsetStart: number,
  offsetSize: number,
  virtualSize: number,
): number {
  const offsetPosition = clientPosition - offsetStart;
  const fractionalPosition = offsetPosition / offsetSize;
  return clamp(Math.round(fractionalPosition * virtualSize), 0, virtualSize - 1);
}

export function calculateLetterboxOffsets(
  sourceWidth: number,
  sourceHeight: number,
  targetWidth: number,
  targetHeight: number,
) {
  const sourceAspectRatio = sourceWidth / sourceHeight;
  const targetAspectRatio = targetWidth / targetHeight;
  let horizontalOffset = 0;
  let verticalOffset = 0;
  let imageWidth = targetWidth;
  let imageHeight = targetHeight;
  if (targetAspectRatio < sourceAspectRatio) {
    // it's letterboxed on the top and bottom
    imageHeight = targetWidth / sourceAspectRatio;
    verticalOffset = (targetHeight - imageHeight) / 2;
  } else if (targetAspectRatio > sourceAspectRatio) {
    // it's letterboxed on the left and right
    imageWidth = targetHeight * sourceAspectRatio;
    horizontalOffset = (targetWidth - imageWidth) / 2;
  }

  return { horizontalOffset, verticalOffset, imageWidth, imageHeight };
}

export function renderCanvas2D(
  canvas: HTMLCanvasElement,
  ctx: CanvasRenderingContext2D,
  sourceWidth: number,
  sourceHeight: number,
  buffer: Uint8Array,
) {
  const { imageWidth, imageHeight, horizontalOffset, verticalOffset } = calculateLetterboxOffsets(
    sourceWidth * 2,
    sourceHeight,
    canvas.offsetWidth,
    canvas.offsetHeight,
  );

  ctx.canvas.width = canvas.offsetWidth;
  ctx.canvas.height = canvas.offsetHeight;
  ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
  const bufferCanvas = new OffscreenCanvas(sourceWidth, sourceHeight);
  const bufferCanvasCtx = bufferCanvas.getContext('2d');
  if (bufferCanvasCtx) {
    const imageData = bufferCanvasCtx.getImageData(0, 0, sourceWidth, sourceHeight);
    imageData.data.set(buffer);
    bufferCanvasCtx.putImageData(imageData, 0, 0);

    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(bufferCanvas, horizontalOffset, verticalOffset, imageWidth, imageHeight);
  }
}
