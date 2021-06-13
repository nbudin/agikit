import React from 'react';
import { EGAPalette } from 'agikit-core/dist/ColorPalettes';
import { stylesForColorNumber } from './colorUtils';
import { usePopoverButton } from './usePopoverButton';

export default function ColorSelector({
  palette,
  color,
  setColor,
  colorType,
  transparentColor,
}: {
  palette: typeof EGAPalette;
  color: number | undefined;
  setColor: React.Dispatch<React.SetStateAction<number | undefined>>;
  colorType?: string;
  transparentColor?: number;
}) {
  const { setButton, setPopover, styles, attributes, open, setOpen } = usePopoverButton();

  return (
    <>
      <button
        ref={setButton}
        type="button"
        className="pic-editor-popover-button secondary"
        onClick={() => setOpen((prevOpen) => !prevOpen)}
        style={stylesForColorNumber(color, palette)}
      >
        {colorType && `${colorType}: `}
        {color ?? 'off'}
      </button>
      <div
        className="pic-editor-color-picker"
        style={{ ...styles.popper, visibility: open ? 'visible' : 'hidden' }}
        ref={setPopover}
        {...attributes.popper}
      >
        <div style={styles.arrow} {...attributes.arrow} />
        <div style={{ display: 'flex', flexWrap: 'wrap' }}>
          {palette.colors.map((_colorValue, colorNumber) => (
            <button
              key={colorNumber}
              type="button"
              className="agikit-tool-button secondary"
              onClick={() => {
                setColor(colorNumber);
              }}
              style={
                colorNumber === transparentColor
                  ? stylesForColorNumber(undefined, palette)
                  : stylesForColorNumber(colorNumber, palette)
              }
            >
              {colorNumber}
            </button>
          ))}
          {transparentColor == null && (
            <button
              type="button"
              className="agikit-tool-button secondary"
              onClick={() => {
                setColor(undefined);
              }}
              style={stylesForColorNumber(undefined, palette)}
            >
              off
            </button>
          )}
        </div>
      </div>
    </>
  );
}
