import React, { useEffect, useState } from 'react';
import { EGAPalette } from 'agikit-core/dist/ColorPalettes';
import { usePopper } from 'react-popper';
import useOnclickOutside from 'react-cool-onclickoutside';
import { stylesForColorNumber } from './colorUtils';

export default function ColorSelector({
  palette,
  color,
  setColor,
  colorType,
}: {
  palette: typeof EGAPalette;
  color: number | undefined;
  setColor: React.Dispatch<React.SetStateAction<number | undefined>>;
  colorType: string;
}) {
  const [button, setButton] = useState<HTMLButtonElement | null>(null);
  const [popover, setPopover] = useState<HTMLDivElement | null>(null);
  const [open, setOpen] = useState(false);
  const { styles, attributes } = usePopper(button, popover);
  const onClickOutsideRef = useOnclickOutside(() => {
    setOpen(false);
  });
  useEffect(() => {
    if (button) {
      onClickOutsideRef(button);
    }
    if (popover) {
      onClickOutsideRef(popover);
    }
  }, [onClickOutsideRef, button, popover]);

  return (
    <>
      <button
        ref={setButton}
        type="button"
        className={`secondary`}
        onClick={() => setOpen((prevOpen) => !prevOpen)}
        style={{
          width: '3rem',
          height: '3rem',
          margin: '1px',
          ...stylesForColorNumber(color, palette),
        }}
      >
        {colorType}: {color ?? 'off'}
      </button>
      <div
        className="pic-editor-color-picker"
        style={{ ...styles.popper, visibility: open ? 'visible' : 'hidden' }}
        ref={setPopover}
        {...attributes.popper}
      >
        <div style={styles.arrow} {...attributes.arrow} />
        <div style={{ display: 'flex', flexWrap: 'wrap' }}>
          {palette.map((_colorValue, colorNumber) => (
            <button
              type="button"
              className="secondary"
              onClick={() => {
                setColor(colorNumber);
              }}
              style={{
                width: '3rem',
                height: '3rem',
                margin: '1px',
                ...stylesForColorNumber(colorNumber, palette),
              }}
            >
              {colorNumber}
            </button>
          ))}
          <button
            type="button"
            className="secondary"
            onClick={() => {
              setColor(undefined);
            }}
            style={{
              width: '3rem',
              height: '3rem',
              margin: '1px',
              ...stylesForColorNumber(undefined, palette),
            }}
          >
            off
          </button>
        </div>
      </div>
    </>
  );
}
