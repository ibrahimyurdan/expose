import type { PointerEvent } from "react";

import { GripGlyph } from "@/components/art/glyphs";
import { bridge } from "@/lib/bridge";

// the corner grip of the frameless window on windows. its edges resize too;
// the grip is the visible, easy target.
export function WindowEdge() {
  const resize = (event: PointerEvent) => {
    if (event.button === 0) {
      event.preventDefault();
      bridge.startResize().catch(() => {});
    }
  };
  return (
    <span className="grip" onPointerDown={resize} aria-hidden>
      <GripGlyph />
    </span>
  );
}
