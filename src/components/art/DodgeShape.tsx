// the dodge button's shape: a flat top, sides that slant out to corners at
// 72% of the height, and a bottom that bows down to the full height in the
// middle, with a red rim. the rim and fill colors come from custom properties
// the .dodge states set (--dodge-rim-top, --dodge-rim-bottom,
// --dodge-fill-top, --dodge-fill-bottom), so css drives hover, press and
// disabled.

import { useId } from "react";

const SHAPE = "M13 0H115L126.3 26Q64 46 1.7 26Z";

export function DodgeShape({ className }: { className?: string }) {
  const id = useId();
  const rim = `dodge-rim-${id}`;
  const fill = `dodge-fill-${id}`;
  const clip = `dodge-clip-${id}`;
  return (
    <svg viewBox="0 0 128 36" width="128" height="36" className={className} aria-hidden focusable={false}>
      <defs>
        <linearGradient id={rim} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" style={{ stopColor: "var(--dodge-rim-top, #f1564b)" }} />
          <stop offset="1" style={{ stopColor: "var(--dodge-rim-bottom, #a60821)" }} />
        </linearGradient>
        <linearGradient id={fill} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" style={{ stopColor: "var(--dodge-fill-top, #23262c)" }} />
          <stop offset=".33" style={{ stopColor: "var(--dodge-fill-mid1, var(--dodge-fill-top, #23262c))" }} />
          <stop offset=".66" style={{ stopColor: "var(--dodge-fill-mid2, var(--dodge-fill-bottom, #1e2328))" }} />
          <stop offset="1" style={{ stopColor: "var(--dodge-fill-bottom, #1e2328)" }} />
        </linearGradient>
        <clipPath id={clip}>
          <path d={SHAPE} />
        </clipPath>
      </defs>
      {/* a dark edge just outside the shape */}
      <path d={SHAPE} fill="none" stroke="#010a13" strokeOpacity=".4" strokeWidth="2" />
      <path d={SHAPE} fill={`url(#${fill})`} />
      {/* the rim is stroked twice as wide and clipped, so it sits inside */}
      <path d={SHAPE} fill="none" stroke={`url(#${rim})`} strokeWidth="4" clipPath={`url(#${clip})`} />
    </svg>
  );
}
