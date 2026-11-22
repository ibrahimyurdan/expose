// expose's mark, the app icon's two nested diamonds: a gold ring around a
// solid gold center. the ring takes the gold gradient, bright at the top;
// below 14px the gradient would muddy, so small marks are flat gold.

import { useId } from "react";

export function Mark({ size = 18, flat, className }: { size?: number; flat?: boolean; className?: string }) {
  const id = useId();
  const ring = `mark-ring-${id}`;
  const plain = flat ?? size < 14;
  return (
    <svg
      viewBox="0 0 18 18"
      width={size}
      height={size}
      className={className}
      aria-hidden
      focusable={false}
    >
      {!plain && (
        <defs>
          <linearGradient id={ring} x1="0" y1="1" x2="0" y2="0">
            <stop offset=".04" stopColor="#463714" />
            <stop offset=".23" stopColor="#785a28" />
            <stop offset=".9" stopColor="#c89b3c" />
            <stop offset="1" stopColor="#c8aa6e" />
          </linearGradient>
        </defs>
      )}
      <path
        fill={plain ? "#c8aa6e" : `url(#${ring})`}
        fillRule="evenodd"
        d="M9 .5 17.5 9 9 17.5.5 9ZM9 2.03 15.97 9 9 15.97 2.03 9Z"
      />
      <path fill="#c8aa6e" d="M9 5.09 12.91 9 9 12.91 5.09 9Z" />
    </svg>
  );
}
