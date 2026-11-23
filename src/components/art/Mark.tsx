// expose's mark, the molded eye of the app icon (src-tauri/icons/source.svg):
// a six-sided eye cut in flat gold faces, lit from the top left, with an
// eight-sided iris and a teal center, a brow above and a chin below. under
// 32 px the faces, brow and chin would blur, so a small mark is the eye
// alone in two tones. a color draws the whole mark flat in it (the grey
// silhouette of an empty state). drawn on the icon's own 1024 grid.

import { useId } from "react";

const EYE = "M196 512 360 382 664 382 828 512 664 642 360 642Z";
const SOCKET = "M284 512 400 438 624 438 740 512 624 586 400 586Z";
const FACE_TOP_LEFT = "M196 512 360 382 512 382 512 438 400 438 284 512Z";
const FACE_TOP_RIGHT = "M828 512 664 382 512 382 512 438 624 438 740 512Z";
const FACE_BOTTOM_LEFT = "M196 512 360 642 512 642 512 586 400 586 284 512Z";
const FACE_BOTTOM_RIGHT = "M828 512 664 642 512 642 512 586 624 586 740 512Z";
const TOP_HALF = "M196 512 360 382 664 382 828 512 740 512 624 438 400 438 284 512Z";
const BOTTOM_HALF = "M196 512 360 642 664 642 828 512 740 512 624 586 400 586 284 512Z";
const IRIS = "M512 410 584 440 614 512 584 584 512 614 440 584 410 512 440 440Z";
const IRIS_CENTER = "M512 452 554 470 572 512 554 554 512 572 470 554 452 512 470 470Z";
const IRIS_QUARTERS = [
  ["M512 410 440 440 410 512 512 512Z", "#f0d79c"],
  ["M512 410 584 440 614 512 512 512Z", "#dcbf80"],
  ["M410 512 440 584 512 614 512 512Z", "#a37b36"],
  ["M614 512 584 584 512 614 512 512Z", "#7a5a22"],
] as const;
const PUPIL = "M512 486 538 512 512 538 486 512Z";
const GLINT = "M488 474 500 466 506 478 494 486Z";
const BROW = "M512 300 600 340 512 326 424 340Z";
const CHIN = "M512 724 600 684 512 698 424 684Z";
const LINE = "#241a0a";

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
  const socket = `mark-socket-${id}`;
  const iris = `mark-iris-${id}`;
  const detailed = !(flat ?? size < 32);

  return (
    <svg
      viewBox="182 182 660 660"
      width={size}
      height={size}
      className={className}
      aria-hidden
      focusable={false}
    >
      {color !== undefined ? (
        // one color, the socket and the iris center cut out.
        <g fill={color} fillRule="evenodd">
          <path d={`${EYE}${SOCKET}`} />
          <path d={`${IRIS}${IRIS_CENTER}`} />
          <path d={PUPIL} />
          {detailed ? (
            <>
              <path d={BROW} />
              <path d={CHIN} />
            </>
          ) : null}
        </g>
      ) : (
        <>
          <defs>
            <radialGradient id={socket} cx="50%" cy="45%" r="60%">
              <stop offset="0" stopColor="#0c2836" />
              <stop offset="1" stopColor="#010a13" />
            </radialGradient>
            <radialGradient id={iris} cx="50%" cy="42%" r="60%">
              <stop offset="0" stopColor="#eafffd" />
              <stop offset=".35" stopColor="#4fe3d4" />
              <stop offset=".8" stopColor="#0a8f9a" />
              <stop offset="1" stopColor="#05505a" />
            </radialGradient>
          </defs>
          {detailed ? (
            <g stroke={LINE} strokeWidth="6" strokeLinejoin="miter">
              <path d={FACE_TOP_LEFT} fill="#f4e2b8" />
              <path d={FACE_TOP_RIGHT} fill="#dcbf80" />
              <path d={FACE_BOTTOM_LEFT} fill="#9a7230" />
              <path d={FACE_BOTTOM_RIGHT} fill="#6e4f1e" />
              <path d={SOCKET} fill={`url(#${socket})`} />
              {IRIS_QUARTERS.map(([d, fill]) => (
                <path key={d} d={d} fill={fill} />
              ))}
              <path d={IRIS_CENTER} fill={`url(#${iris})`} />
              <path d={PUPIL} fill="#041b22" stroke="none" />
              <path d={GLINT} fill="#ffffff" stroke="none" opacity=".85" />
              <path d={BROW} fill="#e8cf94" />
              <path d={CHIN} fill="#8a6528" />
            </g>
          ) : (
            // small: two tones split at the eye's middle, the iris and its
            // teal center, no outlines to blur.
            <g>
              <path d={TOP_HALF} fill="#e2c88f" />
              <path d={BOTTOM_HALF} fill="#9a7230" />
              <path d={SOCKET} fill="#010a13" />
              <path d={IRIS} fill="#c8aa6e" />
              <path d={IRIS_CENTER} fill="#2fd4c6" />
            </g>
          )}
        </>
      )}
    </svg>
  );
}
