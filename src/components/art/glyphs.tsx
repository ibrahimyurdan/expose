// expose's small glyphs, drawn for the league look: window controls, notice
// and match marks, links, the dropdown chevrons. each draws in currentColor
// and is hidden from screen readers; the control around it carries the label.

import { useId, type SVGProps } from "react";

type GlyphProps = SVGProps<SVGSVGElement>;

function glyph(viewBox: string, props: GlyphProps) {
  return { viewBox, "aria-hidden": true, focusable: false, ...props } as const;
}

export function MinimizeGlyph(props: GlyphProps) {
  return (
    <svg {...glyph("0 0 18 18", props)}>
      <rect x="4" y="12" width="10" height="2" fill="currentColor" />
    </svg>
  );
}

export function CloseGlyph(props: GlyphProps) {
  return (
    <svg {...glyph("0 0 18 18", props)}>
      <path d="M4.7 4.7l8.6 8.6m0-8.6l-8.6 8.6" fill="none" stroke="currentColor" strokeWidth="2" />
    </svg>
  );
}

export function GearGlyph(props: GlyphProps) {
  return (
    <svg {...glyph("0 0 18 18", props)}>
      <g fill="currentColor">
        <path fillRule="evenodd" d="M4 9a5 5 0 1 0 10 0a5 5 0 1 0-10 0ZM6 9a3 3 0 1 0 6 0a3 3 0 1 0-6 0Z" />
        {[0, 90, 180, 270].map((angle) => (
          <path key={angle} d="M8 3h2v2H8z" transform={`rotate(${angle} 9 9)`} />
        ))}
        {[45, 135, 225, 315].map((angle) => (
          <path key={angle} d="M8 2.75h2v2.35H8z" transform={`rotate(${angle} 9 9)`} />
        ))}
      </g>
    </svg>
  );
}

// the corner grip that resizes a frameless window on windows.
export function GripGlyph(props: GlyphProps) {
  return (
    <svg {...glyph("0 0 18 18", props)}>
      <g fill="currentColor">
        <rect x="12" y="4" width="2" height="2" />
        <rect x="8" y="8" width="2" height="2" />
        <rect x="12" y="8" width="2" height="2" />
        <rect x="4" y="12" width="2" height="2" />
        <rect x="8" y="12" width="2" height="2" />
        <rect x="12" y="12" width="2" height="2" />
      </g>
    </svg>
  );
}

// a rounded triangle with a cut-out "!", for flags, verdicts and errors.
export function WarnGlyph(props: GlyphProps) {
  return (
    <svg {...glyph("0 0 20 20", props)}>
      <path
        fill="currentColor"
        fillRule="evenodd"
        d="M9.3 3.42Q10 2.2 10.7 3.42L17.9 15.98Q18.6 17.2 17.2 17.2H2.8Q1.4 17.2 2.1 15.98ZM9 6.6H11L10.65 12.2H9.35ZM8.85 14.6a1.15 1.15 0 1 0 2.3 0a1.15 1.15 0 1 0-2.3 0Z"
      />
    </svg>
  );
}

// "looks fine" and a clear verdict.
export function FineGlyph(props: GlyphProps) {
  return (
    <svg {...glyph("0 0 12 12", props)}>
      <path d="M1.6 6.4 4.7 9.5 10.6 2.6" fill="none" stroke="currentColor" strokeWidth="1.8" />
    </svg>
  );
}

// an exact name match.
export function ExactGlyph(props: GlyphProps) {
  return (
    <svg {...glyph("0 0 14 11", props)}>
      <path d="M.5 5.5 4.2 9.5 13.5.5" fill="none" stroke="currentColor" strokeWidth="1.8" />
    </svg>
  );
}

// a link that leaves the app.
export function ExternalGlyph(props: GlyphProps) {
  return (
    <svg {...glyph("0 0 9 9", props)}>
      <path
        d="M3.6 1.6H1.1V7.9H7.4V5.4M4.6 4.4 8.4.6M5.1.6H8.4V3.9"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.2"
      />
    </svg>
  );
}

export function SearchGlyph(props: GlyphProps) {
  return (
    <svg {...glyph("0 0 14 14", props)}>
      <circle cx="5.9" cy="5.9" r="4.15" fill="none" stroke="currentColor" strokeWidth="1.7" />
      <path d="M8.9 8.9 12.9 12.9" fill="none" stroke="currentColor" strokeWidth="2.2" />
    </svg>
  );
}

export function ClockGlyph(props: GlyphProps) {
  return (
    <svg {...glyph("0 0 14 14", props)}>
      <circle cx="7" cy="7" r="5.2" fill="none" stroke="currentColor" strokeWidth="1.5" />
      <path d="M7 3.9V7.3L9.3 8.7" fill="none" stroke="currentColor" strokeWidth="1.5" />
    </svg>
  );
}

// the framed dropdown's up and down chevrons.
export function UpDownGlyph(props: GlyphProps) {
  return (
    <svg {...glyph("0 0 13 18", props)}>
      <path d="M4 6.6 6.5 4.1 9 6.6M4 11.4 6.5 13.9 9 11.4" fill="none" stroke="currentColor" strokeWidth="1.1" />
    </svg>
  );
}

// the checkbox's check: a dark under-stroke keeps it crisp on the gold box.
export function CheckGlyph(props: GlyphProps) {
  return (
    <svg {...glyph("0 0 14 14", props)}>
      <path d="M3.6 6.6 6.2 9.2 12.9 1.6" fill="none" stroke="#010a13" strokeOpacity=".6" strokeWidth="2.6" />
      <path d="M3.6 6.6 6.2 9.2 12.9 1.6" fill="none" stroke="currentColor" strokeWidth="1.6" />
    </svg>
  );
}

// a radio choice: a gold diamond ring; when chosen, a dark field and a cream
// diamond in the middle. the ring's colors come from the --radio-band-* custom
// properties, so hover, pressed and disabled can restyle it from css.
export function RadioGlyph({ checked, ...props }: GlyphProps & { checked: boolean }) {
  const id = useId();
  const band = `radio-band-${id}`;
  const stops: [string, string, string][] = [
    ["0.03", "--radio-band-1", "#c39f56"],
    ["0.16", "--radio-band-2", "#b18730"],
    ["0.5", "--radio-band-3", "#a17d31"],
    ["0.78", "--radio-band-4", "#977530"],
    ["0.97", "--radio-band-5", "#6f5325"],
  ];
  return (
    <svg {...glyph("0 0 18 18", props)}>
      <defs>
        <linearGradient id={band} x1="0" y1="0" x2="0" y2="1">
          {stops.map(([offset, name, color]) => (
            <stop key={offset} offset={offset} style={{ stopColor: `var(${name}, ${color})` }} />
          ))}
        </linearGradient>
      </defs>
      <path
        fill={`url(#${band})`}
        fillRule="evenodd"
        d="M9 1.85 16.15 9 9 16.15 1.85 9ZM9 4.65 13.35 9 9 13.35 4.65 9Z"
      />
      {checked ? (
        <>
          <path d="M9 4.65 13.35 9 9 13.35 4.65 9Z" fill="#010a13" fillOpacity=".7" />
          <path d="M9 6.4 11.6 9 9 11.6 6.4 9Z" style={{ fill: "var(--radio-center, #f0e6d2)" }} />
        </>
      ) : (
        <>
          <path d="M9 4.65 13.35 9 9 13.35 4.65 9Z" fill="#010a13" fillOpacity=".25" />
          <path d="M9 5.2 12.8 9 9 12.8 5.2 9Z" fill="none" stroke="#010a13" strokeOpacity=".44" />
        </>
      )}
    </svg>
  );
}
