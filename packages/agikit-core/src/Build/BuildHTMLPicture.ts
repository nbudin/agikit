import { EGAPalette } from '../ColorPalettes';
import { RenderedPicture } from '../Extract/Picture/RenderPicture';

export function packPictureBuffer(buffer: Uint8Array): string {
  if (buffer.length % 2 != 0) {
    throw new Error('Buffer must have an even number of elements');
  }

  const packedBuffer = Buffer.alloc(buffer.length / 2);
  let offset = 0;
  while (offset < buffer.length) {
    const highNybble = buffer[offset];
    const lowNybble = buffer[offset + 1];
    packedBuffer.writeUInt8((highNybble << 4) + lowNybble, offset / 2);

    offset += 2;
  }

  return packedBuffer.toString('base64');
}

export function writeHTMLPicture(renderedPicture: RenderedPicture): string {
  return `
    <!DOCTYPE html>
    <html>
      <head>
        <script type="text/javascript">
          const EGA_PALETTE = ${JSON.stringify(EGAPalette)};

          function setPixel(ctx, x, y, color) {
            ctx.fillStyle = EGA_PALETTE[color];
            ctx.fillRect(x * 4, y * 2, 4, 2);
          }

          function drawPicture(packedData, canvas) {
            const dataString = atob(packedData);
            const ctx = canvas.getContext('2d');

            for (let index = 0; index < dataString.length; index++) {
              const firstPixelIndex = index * 2;
              const charCode = dataString.charCodeAt(index);
              const highNybble = (charCode & 0xF0) >> 4;
              const lowNybble = charCode & 0x0F;
              setPixel(ctx, firstPixelIndex % 160, Math.floor(firstPixelIndex / 160), highNybble);
              setPixel(ctx, (firstPixelIndex + 1) % 160, Math.floor((firstPixelIndex + 1) / 160), lowNybble);
            }
          }

          const PICTURE_DATA = ${JSON.stringify({
            visual: packPictureBuffer(renderedPicture.visualBuffer),
            priority: packPictureBuffer(renderedPicture.priorityBuffer),
          })};

          document.addEventListener("DOMContentLoaded", function() {
            drawPicture(PICTURE_DATA.visual, document.getElementById('visual'));
            drawPicture(PICTURE_DATA.priority, document.getElementById('priority'));
          });
        </script>
      </head>
      <body>
        <canvas id="visual" width="640" height="400"></canvas>
        <canvas id="priority" width="640" height="400"></canvas>
      </body>
    </html>
  `;
}
