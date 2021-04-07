import { useEffect, useState } from 'react';
import { usePopper } from 'react-popper';
import useOnclickOutside from 'react-cool-onclickoutside';

export function usePopoverButton() {
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

  return { setButton, setPopover, styles, attributes, open, setOpen };
}
