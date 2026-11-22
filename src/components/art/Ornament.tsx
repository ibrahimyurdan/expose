// the welcome title's ornament, sitting under the title: two mirrored halves
// meeting at a small gap. each is a gold bar fading out toward the edge,
// with a bracket at its inner end.

import { useId } from "react";

function Half({ mirrored }: { mirrored?: boolean }) {
  const id = useId();
  const fade = `ornament-fade-${id}`;
  return (
    <svg
      viewBox="0 0 120 8"
      width="120"
      height="8"
      style={mirrored ? { transform: "scaleX(-1)" } : undefined}
      aria-hidden
      focusable={false}
    >
      <defs>
        <linearGradient id={fade} x1="0" y1="0" x2="1" y2="0">
          <stop offset="0" stopColor="#5a4622" stopOpacity="0" />
          <stop offset="1" stopColor="#5a4622" />
        </linearGradient>
      </defs>
      <rect x="0" y="1" width="120" height="2.2" fill={`url(#${fade})`} />
      <rect x="98" y="5.5" width="22" height="1.5" fill="#5a4622" />
      <rect x="118.5" y="1" width="1.5" height="6" fill="#5a4622" />
    </svg>
  );
}

export function Ornament({ className }: { className?: string }) {
  return (
    <div className={className} style={{ display: "flex", justifyContent: "center", gap: 12 }} aria-hidden>
      <Half />
      <Half mirrored />
    </div>
  );
}
