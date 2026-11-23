import { useId, useState } from "react";

import { LeagueArt } from "@/components/art/LeagueArt";
import { PositionGlyph } from "@/components/art/PositionGlyph";
import { roleFile } from "@/lib/art";
import { cn } from "@/lib/utils";
import type { ChampionData, Position } from "@/types";

// the ring's gradient, top to bottom: gold for a teammate, bright gold for
// you, teal while that teammate is picking or banning.
const RINGS = {
  teammate: ["#c8aa6e", "#6e5325"],
  self: ["#f0e6d2", "#c89b3c"],
  acting: ["#cdfafa", "#3097c6"],
} as const;

export type RingTone = keyof typeof RINGS;

// the data dragon url of a champion's square portrait.
function championIconUrl(version: string, imageId: string): string {
  return `https://ddragon.leagueoflegends.com/cdn/${version}/img/champion/${imageId}.png`;
}

// the champion's display name, or null when there is no pick yet or the table
// has not loaded.
export function championName(champions: ChampionData | null, championId: number): string | null {
  if (championId <= 0) {
    return null;
  }
  return champions?.champions[String(championId)]?.name ?? null;
}

// a champion's picture. portrait: the round one in a gold ring, for the rows;
// with no pick yet, a dark disc with the slot's position. square: the small
// one in the recent games list. the name is always written next to it, so
// the image is decorative.
export function ChampionIcon({
  championId,
  champions,
  size,
  ring = "teammate",
  position = null,
  className,
}: {
  championId: number;
  champions: ChampionData | null;
  size: "portrait" | "square";
  ring?: RingTone;
  position?: Position | null;
  className?: string;
}) {
  const entry = championId > 0 ? champions?.champions[String(championId)] : undefined;
  const url = entry && champions ? championIconUrl(champions.version, entry.id) : null;
  // keyed by url, so a new pick starts its own fade-in.
  const picture = url ? <Picture key={url} url={url} /> : null;

  if (size === "square") {
    return (
      <span aria-hidden className={cn("champ-square", className)}>
        {picture}
      </span>
    );
  }
  return (
    <span aria-hidden className={cn("portrait", className)}>
      {championId > 0 ? (
        <span className="portrait-face">{picture}</span>
      ) : (
        <span className="portrait-empty">
          {position ? (
            <LeagueArt
              file={roleFile(position)}
              className="portrait-role"
              fallback={<PositionGlyph position={position} className="portrait-role" />}
            />
          ) : null}
        </span>
      )}
      <Ring tone={ring} />
    </span>
  );
}

function Ring({ tone }: { tone: RingTone }) {
  const id = useId();
  const gradient = `ring-${id}`;
  const [top, bottom] = RINGS[tone];
  return (
    <svg className="portrait-ring" viewBox="0 0 44 44" overflow="visible" focusable={false}>
      <defs>
        <linearGradient id={gradient} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor={top} />
          <stop offset="1" stopColor={bottom} />
        </linearGradient>
      </defs>
      <circle cx="22" cy="22" r="22.5" fill="none" stroke="#010a13" strokeOpacity=".25" />
      <circle cx="22" cy="22" r="21" fill="none" stroke={`url(#${gradient})`} strokeWidth="2" />
      <circle cx="22" cy="22" r="19.5" fill="none" stroke="#010a13" strokeOpacity=".25" />
    </svg>
  );
}

function Picture({ url }: { url: string }) {
  const [state, setState] = useState<"loading" | "loaded" | "failed">("loading");
  if (state === "failed") {
    return null;
  }
  return (
    <img
      src={url}
      alt=""
      draggable={false}
      decoding="async"
      onLoad={() => setState("loaded")}
      onError={() => setState("failed")}
      className={cn(
        "transition-opacity duration-300",
        state === "loaded" ? "opacity-100" : "opacity-0",
      )}
    />
  );
}
