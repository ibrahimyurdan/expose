// the five positions and fill, drawn on a 34 unit grid in two tones: the part
// that names the position in gold, the rest of the map behind it dimmed.

import type { Position } from "@/types";

const ON = "currentColor";
const OFF = { fill: "currentColor", opacity: 0.35 };

// the map's two corner lanes and its middle square, the parts most glyphs share.
const TOP_LANE = "M3 3H25L19 9H9V19L3 25Z";
const BOTTOM_LANE = "M31 31H9L15 25H25V15L31 9Z";
const MIDDLE = "M13 13H21V21H13Z";
const MID_LANE = "M24 4H30V10L10 30H4V24Z";

export function PositionGlyph({ position, className }: { position: Position | "fill"; className?: string }) {
  return (
    <svg viewBox="0 0 34 34" className={className} aria-hidden focusable={false}>
      {position === "top" && (
        <>
          <path d={TOP_LANE} fill={ON} />
          <path d={MIDDLE} {...OFF} />
          <path d={BOTTOM_LANE} {...OFF} />
        </>
      )}
      {position === "bottom" && (
        <>
          <path d={BOTTOM_LANE} fill={ON} />
          <path d={MIDDLE} {...OFF} />
          <path d={TOP_LANE} {...OFF} />
        </>
      )}
      {position === "middle" && (
        <>
          <path d={MID_LANE} fill={ON} />
          <path d="M3 3H17L13 7H7V13L3 17Z" {...OFF} />
          <path d="M31 31H17L21 27H27V21L31 17Z" {...OFF} />
        </>
      )}
      {position === "jungle" && (
        // three claws raking down to the right.
        <path
          fill={ON}
          d="M4 5C10 9 12 16 9 25L14 20C15 13 11 8 4 5ZM13 3C19 8 21 16 18 27L23 21C24 13 20 7 13 3ZM22 6C27 11 28 18 26 28L30 22C31 15 28 10 22 6Z"
        />
      )}
      {position === "utility" && (
        // a winged emblem: a diamond between two wings.
        <>
          <path fill={ON} d="M17 9 22 15 17 26 12 15Z" />
          <path fill={ON} d="M10 12 3 9 6 16 11 17ZM24 12 31 9 28 16 23 17Z" />
        </>
      )}
      {position === "fill" && (
        // six arms, for a slot that plays wherever it is needed.
        <g stroke={ON} strokeWidth="3.2" strokeLinecap="square">
          <path d="M17 5V29" />
          <path d="M6.6 11 27.4 23" />
          <path d="M6.6 23 27.4 11" />
        </g>
      )}
    </svg>
  );
}
