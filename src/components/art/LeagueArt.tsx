import { useState, type ReactNode } from "react";

import { bridge } from "@/lib/bridge";

// the images that have loaded this session: a second row with the same crest
// shows the image at once instead of the drawing first. a failure is kept by
// the instance alone, so a row in the next lobby tries again.
const loaded = new Set<string>();

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
  // the address this one has seen load (which redraws it, to read the set
  // above) and the one it has seen fail.
  const [settled, setSettled] = useState<string | null>(null);
  const [failed, setFailed] = useState<string | null>(null);
  const url = file ? bridge.leagueArt(file) : null;
  if (!url || failed === url) {
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
        onError={() => setFailed(url)}
      />
    </>
  );
}
