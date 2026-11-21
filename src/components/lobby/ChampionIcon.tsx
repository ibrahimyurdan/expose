import { useState } from "react";

import { cn } from "@/lib/utils";
import type { ChampionData } from "@/types";

const SIZES = {
  // the recent games list in a player's details.
  sm: "size-5 rounded-xs",
  // the player rows.
  lg: "size-10 rounded-sm",
} as const;

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

// a champion's square portrait. with no pick yet it is an empty dashed frame;
// while the image loads, or if it fails, a plain frame. the name is always
// written next to it, so the image is decorative.
export function ChampionIcon({
  championId,
  champions,
  size = "lg",
  className,
}: {
  championId: number;
  champions: ChampionData | null;
  size?: keyof typeof SIZES;
  className?: string;
}) {
  const entry = championId > 0 ? champions?.champions[String(championId)] : undefined;
  const url = entry && champions ? championIconUrl(champions.version, entry.id) : null;
  const empty = championId <= 0;

  return (
    <span
      aria-hidden
      className={cn(
        "relative block shrink-0 overflow-hidden border",
        empty ? "border-dashed border-border-strong" : "border-border bg-surface-raised",
        SIZES[size],
        className,
      )}
    >
      {/* keyed by url, so a new pick starts its own fade-in. */}
      {url ? <Portrait key={url} url={url} /> : null}
    </span>
  );
}

function Portrait({ url }: { url: string }) {
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
      // data dragon squares carry a thin dark rim; a slight zoom crops it.
      className={cn(
        "size-full scale-110 object-cover transition-opacity duration-300",
        state === "loaded" ? "opacity-100" : "opacity-0",
      )}
    />
  );
}
