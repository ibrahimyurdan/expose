// expose's mark, the eye of the app icon (src-tauri/icons/source.svg), in the
// spirit of league's vision ping: one flat yellow, lids that run out to sharp
// points, a ring for an iris hanging from the top lid, and a diamond pupil in
// its open middle. under 32 px the pupil would blur into the ring, so a small
// mark leaves the middle open. a color draws the whole mark flat in it (the
// grey silhouette of an empty state). drawn on the icon's own 1024 grid.

import { useId } from "react";

// the band between two sharp almonds
const LIDS =
  "M156 512C334 421.3 369.6 316 512 316C654.4 316 690 421.3 868 512C690 602.7 654.4 708 512 708C369.6 708 334 602.7 156 512Z" +
  "M242 512C377 430.88 404 384 512 384C620 384 647 430.88 782 512C647 593.12 620 640 512 640C404 640 377 593.12 242 512Z";
const PUPIL = "M512 398 532 432 512 466 492 432Z";

export function Mark({
  size = 18,
  flat,
  color,
  className,
}: {
  size?: number;
  flat?: boolean;
  color?: string;
  className?: string;
}) {
  const id = useId();
  const open = `mark-open-${id}`;
  const yellow = `mark-yellow-${id}`;
  const detailed = !(flat ?? size < 32);

  return (
    <svg
      viewBox="142 142 740 740"
      width={size}
      height={size}
      className={className}
      aria-hidden
      focusable={false}
    >
      <defs>
        {/* the iris's open middle, cut through the top lid as well */}
        <mask id={open} maskUnits="userSpaceOnUse" x="0" y="0" width="1024" height="1024">
          <rect width="1024" height="1024" fill="#fff" />
          <circle cx="512" cy="432" r="58" fill="#000" />
        </mask>
        {color === undefined ? (
          <linearGradient
            id={yellow}
            x1="0"
            y1="300"
            x2="0"
            y2="724"
            gradientUnits="userSpaceOnUse"
          >
            <stop offset="0" stopColor="#ffdd73" />
            <stop offset=".55" stopColor="#f6c14b" />
            <stop offset="1" stopColor="#e9a02c" />
          </linearGradient>
        ) : null}
      </defs>
      <g fill={color ?? `url(#${yellow})`}>
        <g mask={`url(#${open})`}>
          <path d={LIDS} fillRule="evenodd" />
          {/* the iris, its top on the top of the lids */}
          <circle cx="512" cy="432" r="116" />
        </g>
        {detailed ? <path d={PUPIL} /> : null}
      </g>
    </svg>
  );
}
