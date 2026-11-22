import { useLayoutEffect, useState, type RefObject } from "react";

import { fitCount } from "@/lib/fit";

// how many chips of a one-line list fit, keeping room for a "+N" chip when
// some do not. `list` is the line's list, as wide as the room it has.
// `ruler` holds every chip at its full width, then a "+N" chip, out of sight.
// measured before the first paint, again whenever the list changes size, and
// once the fonts are in, since they change the widths. `items` names what
// the list holds, so a new set is measured too. null until measured.
export function useFit(
  list: RefObject<HTMLElement | null>,
  ruler: RefObject<HTMLElement | null>,
  items: string,
): number | null {
  const [shown, setShown] = useState<number | null>(null);

  useLayoutEffect(() => {
    const line = list.current;
    const chips = ruler.current;
    if (!line || !chips) {
      return;
    }
    let live = true;
    const fit = () => {
      if (!live) {
        return;
      }
      const widths = Array.from(chips.children, (chip) => chip.getBoundingClientRect().width);
      const more = widths.pop() ?? 0;
      const gap = Number.parseFloat(getComputedStyle(line).columnGap) || 0;
      setShown(fitCount(widths, gap, more, line.getBoundingClientRect().width));
    };
    fit();
    const observer = new ResizeObserver(fit);
    observer.observe(line);
    document.fonts.ready.then(fit, () => {});
    return () => {
      live = false;
      observer.disconnect();
    };
  }, [list, ruler, items]);

  return shown;
}
