// the league client's own images, read by rust from league's archive and
// served at league-art (see src-tauri/src/art.rs): the file names for a tier
// or a role, and the address the page asks for them at. only type imports,
// so node runs the tests in tests/ui straight from source.

import type { Platform } from "@/lib/platform";
import type { Position } from "@/types";

// the protocol rust registers.
const SCHEME = "league-art";

const TIERS = new Set([
  "iron",
  "bronze",
  "silver",
  "gold",
  "platinum",
  "emerald",
  "diamond",
  "master",
  "grandmaster",
  "challenger",
]);

// the client's small crest for a tier, or for unranked; drawn for sizes like
// the row's 14px. null for a tier it doesn't have, so the drawing shows.
export function crestFile(tier: string): string | null {
  const name = tier.toLowerCase();
  return TIERS.has(name) || name === "unranked" ? `images/ranked-mini-crests/${name}.svg` : null;
}

// the tier's full emblem. null for unranked or an unknown tier.
export function emblemFile(tier: string): string | null {
  const name = tier.toLowerCase();
  return TIERS.has(name) ? `images/ranked-emblem/emblem-${name}.png` : null;
}

// the client's gold role icon.
export function roleFile(position: Position): string {
  return `svg/position-${position}.svg`;
}

// custom protocols load from <scheme>://localhost on macos and from
// http://<scheme>.localhost on windows. built here rather than with tauri's
// convertFileSrc, which encodes the slashes the rust side matches exactly.
export function artUrl(file: string, platform: Platform): string {
  return platform === "windows"
    ? `http://${SCHEME}.localhost/${file}`
    : `${SCHEME}://localhost/${file}`;
}
