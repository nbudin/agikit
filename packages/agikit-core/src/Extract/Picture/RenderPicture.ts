import { max } from 'lodash';
import { ColorPalette } from '../../ColorPalettes';
import { PictureCoordinate, PicturePenSettings, PictureResource } from '../../Types/Picture';

export type RenderedPicture = {
  visualBuffer: Uint8Array;
  priorityBuffer: Uint8Array;
};

export type PicturePenMask = {
  origin: PictureCoordinate;
  width: number;
  height: number;
  mask: boolean[];
};

// eslint-disable-next-line @typescript-eslint/no-unused-vars
function penMask(templateStrings: TemplateStringsArray): PicturePenMask {
  const [maskDefinition] = templateStrings;
  const lines = maskDefinition.split(/\r?\n/).filter((line) => line.trim().length > 0);
  const height = lines.length;
  const width = max(lines.map((line) => line.length)) ?? lines[0].length;
  const mask: boolean[] = [];
  let origin: PictureCoordinate | undefined;

  lines.forEach((line, y) => {
    for (let x = 0; x < width; x++) {
      const char = x < line.length ? line[x] : ' ';

      if (char === 'X') {
        mask.push(true);
      } else if (char === '*') {
        origin = { x, y };
        mask.push(true);
      } else if (char === ' ') {
        mask.push(false);
      }
    }
  });

  if (!origin) {
    throw new Error('Origin not found in mask definition');
  }

  return { origin, width, height, mask };
}

// from http://agiwiki.sierrahelp.com/index.php?title=AGI_Specifications:_Chapter_7_-_Picture_Resources#ss7.1
export const penMasks: Record<PicturePenSettings['shape'], { [size: number]: PicturePenMask }> = {
  rectangle: [
    penMask`*`,
    penMask`
XX
X*
XX`,
    penMask`
XXX
XXX
X*X
XXX
XXX`,
    penMask`
XXXX
XXXX
XXXX
XX*X
XXXX
XXXX
XXXX`,
    penMask`
XXXXX
XXXXX
XXXXX
XXXXX
XX*XX
XXXXX
XXXXX
XXXXX
XXXXX`,
    penMask`
XXXXXX
XXXXXX
XXXXXX
XXXXXX
XXXXXX
XXX*XX
XXXXXX
XXXXXX
XXXXXX
XXXXXX
XXXXXX`,
    penMask`
XXXXXXX
XXXXXXX
XXXXXXX
XXXXXXX
XXXXXXX
XXXXXXX
XXX*XXX
XXXXXXX
XXXXXXX
XXXXXXX
XXXXXXX
XXXXXXX
XXXXXXX`,
    penMask`
XXXXXXXX
XXXXXXXX
XXXXXXXX
XXXXXXXX
XXXXXXXX
XXXXXXXX
XXXXXXXX
XXXX*XXX
XXXXXXXX
XXXXXXXX
XXXXXXXX
XXXXXXXX
XXXXXXXX
XXXXXXXX
XXXXXXXX`,
  ],
  circle: [
    penMask`*`,
    penMask`
XX
X*
XX`,
    penMask`
 X
XXX
X*X
XXX
 X `,
    penMask`
 XX
 XX
XXXX
XX*X
XXXX
 XX
 XX `,
    penMask`
  X
 XXX
XXXXX
XXXXX
XX*XX
XXXXX
XXXXX
 XXX
  X  `,
    penMask`
  XX
 XXXX
 XXXX
 XXXX
XXXXXX
XXX*XX
XXXXXX
 XXXX
 XXXX
 XXXX
  XX  `,
    penMask`
  XXX
 XXXXX
 XXXXX
 XXXXX
XXXXXXX
XXXXXXX
XXX*XXX
XXXXXXX
XXXXXXX
 XXXXX
 XXXXX
 XXXXX
  XXX  `,
    penMask`
   XX
  XXXX
 XXXXXX
 XXXXXX
 XXXXXX
XXXXXXXX
XXXXXXXX
XXXX*XXX
XXXXXXXX
XXXXXXXX
 XXXXXX
 XXXXXX
 XXXXXX
  XXXX
   XX`,
  ],
};

// from http://agiwiki.sierrahelp.com/index.php?title=AGI_Specifications:_Chapter_7_-_Picture_Resources#ss7.1
export const penTexturePatterns = Uint8Array.of(
  0x20,
  0x94,
  0x02,
  0x24,
  0x90,
  0x82,
  0xa4,
  0xa2,
  0x82,
  0x09,
  0x0a,
  0x22,
  0x12,
  0x10,
  0x42,
  0x14,
  0x91,
  0x4a,
  0x91,
  0x11,
  0x08,
  0x12,
  0x25,
  0x10,
  0x22,
  0xa8,
  0x14,
  0x24,
  0x00,
  0x50,
  0x24,
  0x04,
);

// from http://agiwiki.sierrahelp.com/index.php?title=AGI_Specifications:_Chapter_7_-_Picture_Resources#ss7.1
export const penTextureStartPositions = Uint8Array.of(
  0x00,
  0x18,
  0x30,
  0xc4,
  0xdc,
  0x65,
  0xeb,
  0x48,
  0x60,
  0xbd,
  0x89,
  0x04,
  0x0a,
  0xf4,
  0x7d,
  0x6d,
  0x85,
  0xb0,
  0x8e,
  0x95,
  0x1f,
  0x22,
  0x0d,
  0xdf,
  0x2a,
  0x78,
  0xd5,
  0x73,
  0x1c,
  0xb4,
  0x40,
  0xa1,
  0xb9,
  0x3c,
  0xca,
  0x58,
  0x92,
  0x34,
  0xcc,
  0xce,
  0xd7,
  0x42,
  0x90,
  0x0f,
  0x8b,
  0x7f,
  0x32,
  0xed,
  0x5c,
  0x9d,
  0xc8,
  0x99,
  0xad,
  0x4e,
  0x56,
  0xa6,
  0xf7,
  0x68,
  0xb7,
  0x25,
  0x82,
  0x37,
  0x3a,
  0x51,
  0x69,
  0x26,
  0x38,
  0x52,
  0x9e,
  0x9a,
  0x4f,
  0xa7,
  0x43,
  0x10,
  0x80,
  0xee,
  0x3d,
  0x59,
  0x35,
  0xcf,
  0x79,
  0x74,
  0xb5,
  0xa2,
  0xb1,
  0x96,
  0x23,
  0xe0,
  0xbe,
  0x05,
  0xf5,
  0x6e,
  0x19,
  0xc5,
  0x66,
  0x49,
  0xf0,
  0xd1,
  0x54,
  0xa9,
  0x70,
  0x4b,
  0xa4,
  0xe2,
  0xe6,
  0xe5,
  0xab,
  0xe4,
  0xd2,
  0xaa,
  0x4c,
  0xe3,
  0x06,
  0x6f,
  0xc6,
  0x4a,
  0x75,
  0xa3,
  0x97,
  0xe1,
);

export const DEFAULT_PEN_SETTINGS: PicturePenSettings = {
  shape: 'rectangle',
  size: 0,
  splatter: false,
};

// ported from http://agiwiki.sierrahelp.com/index.php?title=Picture_Resource_(AGI)
function directionBiasedRound(number: number, direction: number): number {
  if (direction < 0) {
    return number - Math.floor(number) <= 0.501 ? Math.floor(number) : Math.ceil(number);
  } else {
    return number - Math.floor(number) < 0.499 ? Math.floor(number) : Math.ceil(number);
  }
}

function getPixelColor(buffer: Uint8Array, position: PictureCoordinate, palette: ColorPalette) {
  const startOffset = (position.x + position.y * 160) * 4;
  const colorValue = buffer.subarray(startOffset, startOffset + 4);
  return palette.colors.findIndex(
    ([r, g, b, a]) =>
      r === colorValue[0] && g === colorValue[1] && b === colorValue[2] && a === colorValue[3],
  );
}

function setPixelColor(
  buffer: Uint8Array,
  position: PictureCoordinate,
  color: number,
  palette: ColorPalette,
) {
  const startOffset = (position.x + position.y * 160) * 4;
  buffer.set(palette.colors[color], startOffset);
}

function floodFill(
  startPosition: PictureCoordinate,
  targets: { buffer: Uint8Array; color: number }[],
  checkBuffers: { buffer: Uint8Array; backgroundColor: number }[],
  palette: ColorPalette,
) {
  const queue = [startPosition];
  const visited = makeBuffer(0, palette);

  while (queue.length > 0) {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const currentPosition = queue.shift()!;
    if (getPixelColor(visited, currentPosition, palette) === 1) {
      continue;
    } else {
      setPixelColor(visited, currentPosition, 1, palette);
    }

    const isBackgroundPixel = checkBuffers.every((checkBuffer) => {
      const checkColor = getPixelColor(checkBuffer.buffer, currentPosition, palette);
      return checkColor === checkBuffer.backgroundColor;
    });

    if (isBackgroundPixel) {
      targets.forEach(({ buffer, color }) => {
        setPixelColor(buffer, currentPosition, color, palette);
      });
      if (currentPosition.x > 0) {
        queue.push({ x: currentPosition.x - 1, y: currentPosition.y });
      }
      if (currentPosition.y > 0) {
        queue.push({ x: currentPosition.x, y: currentPosition.y - 1 });
      }
      if (currentPosition.x < 159) {
        queue.push({ x: currentPosition.x + 1, y: currentPosition.y });
      }
      if (currentPosition.y < 167) {
        queue.push({ x: currentPosition.x, y: currentPosition.y + 1 });
      }
    }
  }
}

// ported from http://agiwiki.sierrahelp.com/index.php?title=Picture_Resource_(AGI)
function drawLine(
  buffer: Uint8Array,
  start: PictureCoordinate,
  finish: PictureCoordinate,
  color: number,
  palette: ColorPalette,
) {
  const height = finish.y - start.y;
  const width = finish.x - start.x;
  let addX = height == 0 ? height : width / Math.abs(height);
  let addY = width == 0 ? width : height / Math.abs(width);
  let x: number;
  let y: number;

  if (Math.abs(width) > Math.abs(height)) {
    y = start.y;
    addX = width == 0 ? 0 : width / Math.abs(width);
    for (x = start.x; x != finish.x; x += addX) {
      setPixelColor(
        buffer,
        { x: directionBiasedRound(x, addX), y: directionBiasedRound(y, addY) },
        color,
        palette,
      );
      y += addY;
    }
    setPixelColor(buffer, finish, color, palette);
  } else {
    x = start.x;
    addY = height == 0 ? 0 : height / Math.abs(height);
    for (y = start.y; y != finish.y; y += addY) {
      setPixelColor(
        buffer,
        { x: directionBiasedRound(x, addX), y: directionBiasedRound(y, addY) },
        color,
        palette,
      );
      x += addX;
    }
    setPixelColor(buffer, finish, color, palette);
  }
}

function makeBuffer(fillColor: number, palette: ColorPalette): Uint8Array {
  const colorAsString = Buffer.from(palette.colors[fillColor]).toString('binary');
  return Buffer.alloc(160 * 168 * 4, colorAsString, 'binary');
}

export function renderPicture(
  picture: PictureResource,
  palette: ColorPalette,
  startingFrom?: {
    renderedPicture: RenderedPicture;
    pictureColor: number | undefined;
    priorityColor: number | undefined;
    pen: PicturePenSettings;
  },
): RenderedPicture {
  const visualBuffer = startingFrom
    ? new Uint8Array(startingFrom.renderedPicture.visualBuffer)
    : makeBuffer(15, palette);
  const priorityBuffer = startingFrom
    ? new Uint8Array(startingFrom.renderedPicture.priorityBuffer)
    : makeBuffer(4, palette);
  let pictureColor: number | undefined = startingFrom?.pictureColor;
  let priorityColor: number | undefined = startingFrom?.priorityColor;
  let pen: PicturePenSettings = startingFrom?.pen ?? DEFAULT_PEN_SETTINGS;

  const drawLineInCurrentColors = (start: PictureCoordinate, finish: PictureCoordinate) => {
    if (pictureColor != null) {
      drawLine(visualBuffer, start, finish, pictureColor, palette);
    }
    if (priorityColor != null) {
      drawLine(priorityBuffer, start, finish, priorityColor, palette);
    }
  };

  const floodFillInCurrentColors = (startPosition: PictureCoordinate) => {
    if (pictureColor != null) {
      floodFill(
        startPosition,
        [
          { buffer: visualBuffer, color: pictureColor },
          ...(priorityColor != null ? [{ buffer: priorityBuffer, color: priorityColor }] : []),
        ],
        [{ buffer: visualBuffer, backgroundColor: 15 }],
        palette,
      );
    } else if (priorityColor != null) {
      floodFill(
        startPosition,
        [{ buffer: priorityBuffer, color: priorityColor }],
        [{ buffer: priorityBuffer, backgroundColor: 4 }],
        palette,
      );
    }
  };

  const plotWithPenInCurrentColors = (position: PictureCoordinate, texture: number | undefined) => {
    const penMask = penMasks[pen.shape][pen.size];
    let textureStartPosition: number | undefined;
    let maskOnPixelCount = 0; // texture bitmap only affects masked-on pixels; only count those
    if (pen.splatter && texture != null) {
      textureStartPosition = penTextureStartPositions[texture];
    }

    for (let index = 0; index < penMask.mask.length; index++) {
      const maskOn = penMask.mask[index];
      if (!maskOn) {
        continue;
      }

      if (textureStartPosition != null) {
        // yes, mod 255, per the AGI spec.  Lance Ewing thinks it was a bug in AGI itself
        const texturePosition = (textureStartPosition + maskOnPixelCount) % 255;
        const textureByte = penTexturePatterns[Math.floor(texturePosition / 8)];
        const textureBit = textureByte & (1 << texturePosition % 8);
        maskOnPixelCount += 1;
        if (textureBit === 0) {
          continue;
        }
      }

      const maskX = index % penMask.width;
      const maskY = Math.floor(index / penMask.width);
      const logicalX = maskX - penMask.origin.x;
      const logicalY = maskY - penMask.origin.y;
      const screenX = logicalX + position.x;
      const screenY = logicalY + position.y;
      if (pictureColor != null) {
        setPixelColor(visualBuffer, { x: screenX, y: screenY }, pictureColor, palette);
      }
      if (priorityColor != null) {
        setPixelColor(priorityBuffer, { x: screenX, y: screenY }, priorityColor, palette);
      }
    }
  };

  picture.commands.forEach((command) => {
    if (command.type === 'SetPictureColor') {
      pictureColor = command.colorNumber;
    } else if (command.type === 'SetPriorityColor') {
      priorityColor = command.colorNumber;
    } else if (command.type === 'DisablePictureDraw') {
      pictureColor = undefined;
    } else if (command.type === 'DisablePriorityDraw') {
      priorityColor = undefined;
    } else if (command.type === 'AbsoluteLine') {
      if (command.points.length > 1) {
        let lastPoint = command.points[0];
        command.points.slice(1).forEach((point) => {
          drawLineInCurrentColors(lastPoint, point);
          lastPoint = point;
        });
      }
    } else if (command.type === 'RelativeLine') {
      if (command.relativePoints.length > 0) {
        let lastPoint = command.startPosition;
        command.relativePoints.forEach((relativePoint) => {
          const point = { x: lastPoint.x + relativePoint.x, y: lastPoint.y + relativePoint.y };
          drawLineInCurrentColors(lastPoint, point);
          lastPoint = point;
        });
      }
    } else if (command.type === 'DrawXCorner' || command.type === 'DrawYCorner') {
      if (command.steps.length > 0) {
        let lastPoint = command.startPosition;
        command.steps.forEach((step) => {
          const point = { ...lastPoint };
          point[step.axis] = step.position;
          drawLineInCurrentColors(lastPoint, point);
          lastPoint = point;
        });
      }
    } else if (command.type === 'Fill') {
      command.startPositions.forEach((startPosition) => {
        floodFillInCurrentColors(startPosition);
      });
    } else if (command.type === 'ChangePen') {
      pen = command.settings;
    } else if (command.type === 'PlotWithPen') {
      command.points.forEach((point) => {
        plotWithPenInCurrentColors(point.position, point.texture);
      });
    }
  });

  return { visualBuffer, priorityBuffer };
}
