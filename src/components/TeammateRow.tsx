import { ExternalLink } from "lucide-react";
import { useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { bridge } from "@/lib/bridge";
import { cn } from "@/lib/utils";
import type { ChampionData, Teammate } from "@/types";

const POSITION_LABELS: Record<string, string> = {
  top: "TOP",
  jungle: "JNG",
  middle: "MID",
  bottom: "BOT",
  utility: "SUP",
};

function championIcon(
  champions: ChampionData | null,
  championId: number,
): { url: string; name: string } | null {
  if (!champions || championId <= 0) {
    return null;
  }
  const entry = champions.champions[String(championId)];
  if (!entry) {
    return null;
  }
  return {
    url: `https://ddragon.leagueoflegends.com/cdn/${champions.version}/img/champion/${entry.id}.png`,
    name: entry.name,
  };
}

function profileIconUrl(champions: ChampionData | null, profileIconId: number): string | null {
  if (!champions || profileIconId <= 0) {
    return null;
  }
  return `https://ddragon.leagueoflegends.com/cdn/${champions.version}/img/profileicon/${profileIconId}.png`;
}

export function TeammateRow({
  teammate,
  champions,
}: {
  teammate: Teammate;
  champions: ChampionData | null;
}) {
  const champion = championIcon(champions, teammate.championId);
  const fallback = profileIconUrl(champions, teammate.profileIconId);
  const position = POSITION_LABELS[teammate.assignedPosition.toLowerCase()];

  return (
    <div className="hex-frame flex items-center gap-3 rounded-sm p-2">
      <Avatar champion={champion} fallback={fallback} />
      <div className="min-w-0 flex-1">
        <div className="flex items-baseline gap-1">
          <Tooltip>
            <TooltipTrigger asChild>
              <span className="min-w-0 truncate text-sm font-semibold text-foreground">
                {teammate.gameName}
              </span>
            </TooltipTrigger>
            <TooltipContent>
              {teammate.gameName}#{teammate.tagLine}
            </TooltipContent>
          </Tooltip>
          <span className="shrink-0 text-xs text-muted-foreground">#{teammate.tagLine}</span>
        </div>
        <div className="mt-0.5 flex items-center gap-2">
          {position ? <Badge className="w-9 justify-center">{position}</Badge> : null}
          {champion ? (
            <span className="truncate text-xs font-medium text-gold-bright">{champion.name}</span>
          ) : null}
          {teammate.summonerLevel > 0 ? (
            <span className="ml-auto flex shrink-0 items-baseline gap-1">
              <span className="text-[10px] text-muted-foreground">Lv</span>
              <span className="text-xs font-semibold tabular-nums text-foreground">
                {teammate.summonerLevel}
              </span>
            </span>
          ) : null}
        </div>
      </div>
      <Button
        variant="outline"
        size="sm"
        onClick={() => bridge.openExternal(teammate.scoutUrl).catch(() => {})}
        className="border-gold-deep/40 bg-secondary/40 text-muted-foreground hover:border-gold hover:text-gold-bright"
        aria-label={`Open ${teammate.gameName} on ${teammate.scoutLabel}`}
        title={`Open on ${teammate.scoutLabel}`}
      >
        {teammate.scoutLabel}
        <ExternalLink className="h-3 w-3" aria-hidden />
      </Button>
    </div>
  );
}

function Avatar({
  champion,
  fallback,
}: {
  champion: { url: string; name: string } | null;
  fallback: string | null;
}) {
  const source = champion?.url ?? fallback;
  const [loaded, setLoaded] = useState(false);
  const [errored, setErrored] = useState(false);

  if (!source || errored) {
    return (
      <div
        className="flex h-11 w-11 shrink-0 items-center justify-center rounded-sm border border-gold-deep bg-secondary text-lg leading-none text-hex-teal"
        aria-hidden
      >
        ◆
      </div>
    );
  }

  const image = (
    <img
      src={source}
      alt={champion?.name ?? "summoner icon"}
      onLoad={() => setLoaded(true)}
      onError={() => setErrored(true)}
      className={cn(
        "h-11 w-11 shrink-0 rounded-sm border border-gold-deep object-cover transition-opacity duration-300",
        loaded ? "opacity-100" : "opacity-0",
      )}
    />
  );

  if (!champion) {
    return image;
  }

  return (
    <Tooltip>
      <TooltipTrigger asChild>{image}</TooltipTrigger>
      <TooltipContent>{champion.name}</TooltipContent>
    </Tooltip>
  );
}
