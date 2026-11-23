import { useState, type ReactNode } from "react";

import { bridge } from "@/lib/bridge";

// the images that have loaded, or failed, this session. a second row with the
// same crest shows the image at once instead of the drawing first.
const loaded = new Set<string>();
const broken = new Set<string>();

// one of league's own images (see lib/art.ts), with expose's drawing in its
// place until the image has loaded, and for good when it can't: no league
// install, a file the archive lacks, or the browser preview. the image is
// decorative; the text beside it says what it shows.
export function LeagueArt({
  file,
  className,
  fallback,
}: {
  file: string | null;
  className?: string;
  fallback: ReactNode;
}) {
  // the address this one has seen load or fail: setting it redraws this
  // instance, which then reads the shared sets above.
  const [settled, setSettled] = useState<string | null>(null);
  const url = file ? bridge.leagueArt(file) : null;
  if (!url || broken.has(url)) {
    return <>{fallback}</>;
  }
  const shown = loaded.has(url);
  return (
    <>
      {shown ? null : fallback}
      <img
        src={url}
        alt=""
        aria-hidden
        draggable={false}
        decoding="async"
        className={className}
        hidden={!shown}
        onLoad={() => {
          loaded.add(url);
          if (settled !== url) {
            setSettled(url);
          }
        }}
        onError={() => {
          broken.add(url);
          if (settled !== url) {
            setSettled(url);
          }
        }}
      />
    </>
  );
}
