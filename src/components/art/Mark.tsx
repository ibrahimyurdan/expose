// expose's mark, the app icon's two nested diamonds: a gold ring around a
// solid gold center. the ring takes the gold gradient, bright at the top;
// below 14px the gradient would muddy, so small marks are flat gold. a color
// draws the whole mark flat in it (the grey silhouette of an empty state).

import { useId } from "react";

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
  const ring = `mark-ring-${id}`;
  const plain = color !== undefined || (flat ?? size < 14);
  const fill = color ?? "#c8aa6e";
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
        fill={plain ? fill : `url(#${ring})`}
        fillRule="evenodd"
        d="M9 .5 17.5 9 9 17.5.5 9ZM9 2.03 15.97 9 9 15.97 2.03 9Z"
      />
      <path fill={fill} d="M9 5.09 12.91 9 9 12.91 5.09 9Z" />
    </svg>
  );
}
