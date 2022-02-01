import { CSSProperties } from 'react';
import { EGAPalette } from '@agikit/core';

export function backgroundStylesForColorNumber(
  colorNumber: number | undefined,
  palette: typeof EGAPalette,
): CSSProperties {
  if (colorNumber != null) {
    const color = palette.colors[colorNumber];
    if (color != null) {
      return { backgroundColor: `rgb(${color[0]}, ${color[1]}, ${color[2]})` };
    }
  }

  return {
    background:
      'linear-gradient(45deg, rgba(0, 0, 0, 0.0980392) 25%, transparent 25%, transparent 75%, rgba(0, 0, 0, 0.0980392) 75%, rgba(0, 0, 0, 0.0980392) 0), linear-gradient(45deg, rgba(0, 0, 0, 0.0980392) 25%, transparent 25%, transparent 75%, rgba(0, 0, 0, 0.0980392) 75%, rgba(0, 0, 0, 0.0980392) 0), white',
    backgroundSize: '10px 10px, 10px 10px',
    backgroundPosition: '0 0, 5px 5px',
  };
}

export function textColorForBackgroundColor(backgroundColor: number | undefined): string {
  if (backgroundColor != null && backgroundColor < 10) {
    return 'white';
  }

  return 'black';
}

export function stylesForColorNumber(
  colorNumber: number | undefined,
  palette: typeof EGAPalette,
): CSSProperties {
  return {
    ...backgroundStylesForColorNumber(colorNumber, palette),
    color: textColorForBackgroundColor(colorNumber),
  };
}
