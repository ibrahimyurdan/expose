// the gold metal-work at the outer end of each timer bar: a thin rail, a band
// that steps down into a tab capping the bar, and a line that curls down into
// the base line running under the bar. the right side is the left mirrored.

import { useId } from "react";

export function MetalWork({ side, className }: { side: "left" | "right"; className?: string }) {
  const id = useId();
  const metal = `metal-${id}`;
  return (
    <svg
      viewBox="0 0 48 21"
      width="48"
      height="21"
      className={className}
      style={side === "right" ? { transform: "scaleX(-1)" } : undefined}
      aria-hidden
      focusable={false}
    >
      <defs>
        <linearGradient id={metal} gradientUnits="userSpaceOnUse" x1="0" y1="0" x2="99" y2="0">
          <stop offset="0" stopColor="#463714" />
          <stop offset="1" stopColor="#785a28" />
        </linearGradient>
      </defs>
      <g fill="none" stroke={`url(#${metal})`}>
        <path d="M0 .5H30M29.5 1V3" />
        <path d="M.7 1.3 3.4 4H31C32 11 36.5 15 44 15" strokeWidth="2" />
        <path d="M28.6 7C29.2 15.2 35 20 46 20H48" />
      </g>
      <rect x="44" y="12.5" width="1.6" height="5" fill={`url(#${metal})`} />
    </svg>
  );
}
