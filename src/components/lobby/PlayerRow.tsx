import { BadgeCheck, ChevronDown, CircleCheck } from "lucide-react";
import { useEffect, useId, useRef, type CSSProperties } from "react";

import { ChampionIcon, championName } from "@/components/lobby/ChampionIcon";
import { FormDots } from "@/components/lobby/FormDots";
import { PlayerDetail } from "@/components/lobby/PlayerDetail";
import { RankBadge } from "@/components/lobby/RankBadge";
import { Chip, type ChipTone } from "@/components/ui/chip";
import { IconButton } from "@/components/ui/icon-button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { flagText, formatKda, plural, positionLabel, riotId } from "@/lib/format";
import { cn } from "@/lib/utils";
import type {
  ChampionData,
  Confidence,
  IdentityView,
  MatchKind,
  PlayerView,
  Severity,
  StatsView,
} from "@/types";

const FLAG_TONES: Record<Severity, ChipTone> = {
  red: "red",
  amber: "amber",
  info: "neutral",
};

// a guess the backend is sure of looks settled; a shaky one asks for a second
// look.
const GUESS_TONES: Record<Confidence, ChipTone> = {
  high: "accent",
  medium: "neutral",
  low: "amber",
};

const MATCH_HINTS: Record<Exclude<MatchKind, "own" | "guess">, string> = {
  visible: "Name shown by League",
  key: "Exact match",
  elimination: "Exact match, the only name left",
};

// one teammate (or you), in three lines next to the champion portrait:
//   1. the slot's name (the alias when ranked hides names), the champion and
//      the solo/duo rank;
//   2. who is behind an alias (exact, or a "likely" guess), recent form and kda;
//   3. red flags, or a quiet "looks fine".
// stats fill in as they arrive, with a shimmer until then. a row with stats
// opens to show more; the whole row toggles it, and the chevron button is the
// keyboard and screen reader control.
export function PlayerRow({
  player,
  champions,
  expanded,
  onToggle,
  scoutLabel,
  style,
}: {
  player: PlayerView;
  champions: ChampionData | null;
  expanded: boolean;
  onToggle: () => void;
  scoutLabel: string;
  style?: CSSProperties;
}) {
  const detailId = useId();
  const card = useRef<HTMLElement>(null);
  const { identity, stats } = player;
  // the backend says when a hidden teammate is still being matched, rather
  // than hidden for good.
  const loading = player.statsPending || player.matching;
  const expandable = stats !== null;
  const open = expanded && stats !== null;

  // a row opened near the bottom would unfold out of sight, so bring it into
  // view (its scroll margin keeps it clear of the pinned band above).
  useEffect(() => {
    if (!open) {
      return;
    }
    const still = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    card.current?.scrollIntoView({ block: "nearest", behavior: still ? "auto" : "smooth" });
  }, [open]);

  // what the client shows for the slot: the alias in ranked, else the name.
  // a match can come before its name does, so an empty name falls through.
  const title = player.alias || identity?.gameName || (player.isSelf ? "You" : "Teammate");
  // a name league shows (normal games, your party): the whole riot id, since
  // line two has no room to repeat it.
  const shown = !player.alias && identity && identity.gameName ? identity : null;
  const champion = championName(champions, player.championId);
  const picked = player.championId > 0;

  return (
    <li
      style={style}
      className="animate-in duration-300 fill-mode-backwards fade-in-0 slide-in-from-bottom-1"
    >
      <article
        ref={card}
        aria-label={title}
        className={cn(
          // the pinned band is about 160px tall.
          "scroll-mt-42 scroll-mb-4 rounded-card border bg-surface transition-colors",
          open ? "border-border-strong" : "border-border",
          expandable && !open && "hover:border-border-strong",
        )}
      >
        {/* the chevron below is the real control; the row is a bigger target
            for the mouse. */}
        <div className="flex gap-3 p-3" onClick={expandable ? onToggle : undefined}>
          <div className="flex w-10 shrink-0 flex-col items-center gap-1">
            <ChampionIcon championId={player.championId} champions={champions} />
            <span className="-mx-1 h-4 text-xs whitespace-nowrap text-fg-secondary">
              {player.position ? positionLabel(player.position) : null}
            </span>
          </div>

          <div className="flex min-w-0 flex-1 flex-col gap-1">
            <div className="flex h-5 items-center gap-3">
              <p className="flex min-w-0 flex-1 items-baseline gap-2">
                {shown ? (
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <span className="max-w-[65%] shrink-0 truncate text-base font-semibold text-fg">
                        {shown.gameName}
                        {shown.tagLine ? (
                          <span className="font-normal text-fg-secondary">#{shown.tagLine}</span>
                        ) : null}
                      </span>
                    </TooltipTrigger>
                    <TooltipContent>{riotId(shown)}</TooltipContent>
                  </Tooltip>
                ) : (
                  <span className="max-w-[65%] shrink-0 truncate text-base font-semibold text-fg">
                    {title}
                  </span>
                )}
                <span className="min-w-0 truncate text-base text-fg-secondary">
                  {/* the champion table may still be loading, or not know a
                      brand new champion yet. */}
                  {champion ?? (picked ? "Unknown champion" : "No pick yet")}
                  {picked && stats && stats.championGames !== null ? (
                    <span className="text-sm"> · {plural(stats.championGames, "game")}</span>
                  ) : null}
                </span>
              </p>
              {loading ? (
                <span aria-hidden className="skeleton h-3 w-20 shrink-0" />
              ) : stats ? (
                <RankBadge rank={stats.rank} unranked={stats.unranked} className="shrink-0" />
              ) : null}
            </div>

            <div className="flex h-4 items-center gap-3">
              <div className="flex min-w-0 flex-1 items-center">
                <Identity player={player} loading={loading} />
              </div>
              <Form stats={stats} loading={loading} />
            </div>

            <FlagLine
              player={player}
              loading={loading}
              expandable={expandable}
              expanded={open}
              detailId={detailId}
              onToggle={onToggle}
              title={title}
            />
          </div>
        </div>

        {open ? (
          <PlayerDetail
            id={detailId}
            player={player}
            stats={stats}
            champions={champions}
            scoutLabel={scoutLabel}
          />
        ) : null}
      </article>
    </li>
  );
}

// line two, left: who the row is. "You" on your own row; behind an alias, the
// matched riot id with how sure the match is; on a visible name (normal games,
// where line one already shows it), the account level.
function Identity({ player, loading }: { player: PlayerView; loading: boolean }) {
  const { identity } = player;

  if (player.isSelf) {
    return (
      <Chip tone="accent" className="h-4 px-1.5">
        You
      </Chip>
    );
  }
  if (!player.alias) {
    if (loading) {
      return <span aria-hidden className="skeleton h-3 w-14" />;
    }
    return player.stats?.level != null ? (
      <span className="text-sm text-fg-secondary tabular-nums">Level {player.stats.level}</span>
    ) : null;
  }
  if (!identity) {
    return (
      <span className="truncate text-sm text-fg-secondary">
        {player.matching ? "Matching name…" : "Name hidden"}
      </span>
    );
  }
  return (
    <span className="flex min-w-0 items-center gap-1.5">
      <Tooltip>
        <TooltipTrigger asChild>
          <span className="min-w-0 truncate text-sm text-fg">
            {identity.gameName}
            {identity.tagLine ? <span className="text-fg-secondary">#{identity.tagLine}</span> : null}
          </span>
        </TooltipTrigger>
        <TooltipContent>{riotId(identity)}</TooltipContent>
      </Tooltip>
      <MatchBadge identity={identity} />
    </span>
  );
}

// a guess wears a "likely" chip toned by confidence; an exact match a small
// check. both explain themselves on hover.
function MatchBadge({ identity }: { identity: IdentityView }) {
  if (identity.kind === "own") {
    return null;
  }
  if (identity.kind === "guess") {
    const hint = `Best guess from recent games, ${identity.confidence} confidence`;
    return (
      <Tooltip>
        <TooltipTrigger asChild>
          <Chip tone={GUESS_TONES[identity.confidence]} className="h-4 px-1.5">
            Likely
            <span className="sr-only">, {hint}</span>
          </Chip>
        </TooltipTrigger>
        <TooltipContent>{hint}</TooltipContent>
      </Tooltip>
    );
  }
  const hint = MATCH_HINTS[identity.kind];
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <span className="inline-flex shrink-0 text-accent">
          <BadgeCheck className="size-3.5" aria-hidden />
          <span className="sr-only">{hint}</span>
        </span>
      </TooltipTrigger>
      <TooltipContent>{hint}</TooltipContent>
    </Tooltip>
  );
}

// line two, right: the last ten results and the kda over recent games, in
// fixed widths so they line up from row to row.
function Form({ stats, loading }: { stats: StatsView | null; loading: boolean }) {
  if (loading) {
    return (
      <span aria-hidden className="flex shrink-0 items-center gap-2">
        <span className="skeleton h-2 w-19" />
        <span className="skeleton h-3 w-13" />
      </span>
    );
  }
  if (!stats) {
    return null;
  }
  if (!stats.historyLoaded) {
    return <span className="shrink-0 text-sm text-fg-secondary">No match history</span>;
  }
  if (stats.recent.length === 0) {
    return <span className="shrink-0 text-sm text-fg-secondary">No recent games</span>;
  }
  return (
    <span className="flex shrink-0 items-center gap-2">
      <FormDots games={stats.recent} />
      <span className="w-13 text-right text-sm whitespace-nowrap tabular-nums">
        <span className="text-fg">{stats.kda !== null ? formatKda(stats.kda) : "–"}</span>
        <span className="text-fg-secondary"> KDA</span>
      </span>
    </span>
  );
}

// line three: the flag chips (they wrap), "looks fine", or for your own row the
// account level and season, since you are never flagged. the details toggle
// sits at its end.
function FlagLine({
  player,
  loading,
  expandable,
  expanded,
  detailId,
  onToggle,
  title,
}: {
  player: PlayerView;
  loading: boolean;
  expandable: boolean;
  expanded: boolean;
  detailId: string;
  onToggle: () => void;
  title: string;
}) {
  const { stats, flags } = player;

  let content;
  if (loading) {
    content = <span aria-hidden className="skeleton h-5 w-28 rounded-full!" />;
  } else if (!stats) {
    // nothing known about this player (their name stayed hidden).
    return null;
  } else if (player.isSelf) {
    content = <span className="truncate text-sm text-fg-secondary">{selfSummary(stats)}</span>;
  } else if (flags.length === 0) {
    content = (
      <span className="flex items-center gap-1.5 text-sm text-fg-secondary">
        <CircleCheck className="size-3.5 shrink-0 text-green" aria-hidden />
        Looks fine
      </span>
    );
  } else {
    content = (
      <ul aria-label="Flags" className="flex min-w-0 flex-wrap gap-1">
        {flags.map((flag) => (
          <li key={flag.kind} className="flex min-w-0">
            <Chip tone={FLAG_TONES[flag.severity]} title={flagText(flag.label)}>
              {flagText(flag.label)}
            </Chip>
          </li>
        ))}
      </ul>
    );
  }

  return (
    <div className="flex min-h-5 items-start gap-2">
      <div className="flex min-h-5 min-w-0 flex-1 items-center">{content}</div>
      {expandable ? (
        <IconButton
          label={expanded ? `Hide details for ${title}` : `Show details for ${title}`}
          size="xs"
          aria-expanded={expanded}
          aria-controls={expanded ? detailId : undefined}
          onClick={(event) => {
            // the row toggles on click too; one click must not toggle twice.
            event.stopPropagation();
            onToggle();
          }}
          className="-my-0.5 -mr-1"
        >
          <ChevronDown
            aria-hidden
            className={cn("transition-transform duration-200", expanded && "rotate-180")}
          />
        </IconButton>
      ) : null}
    </div>
  );
}

// your own third line: level and ranked season.
function selfSummary(stats: StatsView): string {
  const parts: string[] = [];
  if (stats.level !== null) {
    parts.push(`Level ${stats.level}`);
  }
  if (stats.rank) {
    parts.push(`${stats.rank.wins}W ${stats.rank.losses}L this season`);
  }
  return parts.length > 0 ? parts.join(" · ") : "Your recent games";
}
