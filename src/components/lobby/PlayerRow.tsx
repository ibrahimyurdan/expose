import { useEffect, useId, useRef, type CSSProperties } from "react";

import { ExactGlyph, FineGlyph } from "@/components/art/glyphs";
import { ChampionIcon, championName, type RingTone } from "@/components/lobby/ChampionIcon";
import { FormDots } from "@/components/lobby/FormDots";
import { PlayerDetail } from "@/components/lobby/PlayerDetail";
import { RankBadge } from "@/components/lobby/RankBadge";
import { Chip, Tag, type ChipTone, type TagTone } from "@/components/ui/chip";
import { Loader } from "@/components/ui/loader";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { useFit } from "@/hooks/useFit";
import { flagText, formatKda, plural, positionLabel, riotId } from "@/lib/format";
import { cn } from "@/lib/utils";
import type {
  ChampionData,
  Confidence,
  Flag,
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
const GUESS_TONES: Record<Confidence, TagTone> = {
  high: "gold",
  medium: "gold",
  low: "amber",
};

const MATCH_HINTS: Record<Exclude<MatchKind, "own" | "guess">, string> = {
  visible: "Name shown by League",
  key: "Exact match",
  elimination: "Exact match, the only name left",
};

// the kda shown green from this and up, red under the low mark (a little
// above the 1.5 that raises a bad kda flag, so a weak kda shows before it is
// a flag).
const KDA_HIGH = 3;
const KDA_LOW = 2;

// one teammate (or you), in four lines next to the champion portrait:
//   1. the slot's role, and the solo/duo rank;
//   2. the champion and the games on it, and the last ten results and kda;
//   3. the alias and who is behind it (exact, or a "likely" guess);
//   4. red flags, or a quiet "looks fine", and the details toggle.
// every line keeps its height, so the row does not move as stats arrive. a
// row with stats opens to show more; the whole row toggles it, and the
// triangle button is the keyboard and screen reader control.
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
  const entry = useRef<HTMLLIElement>(null);
  const { identity, stats } = player;
  // the backend says when a hidden teammate is still being matched, rather
  // than hidden for good.
  const matching = player.matching;
  // the summoner and rank can be in while the history is still on the way.
  const historyPending = player.statsPending || matching;
  const expandable = stats !== null;
  const open = expanded && stats !== null;

  // the details are taller than the room under the pinned band, so an opened
  // row moves up to just under the band, with as much of its details as fits
  // below it (the entry's scroll margin keeps it clear of the band).
  useEffect(() => {
    if (!open) {
      return;
    }
    const still = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    entry.current?.scrollIntoView({ block: "start", behavior: still ? "auto" : "smooth" });
  }, [open]);

  // what the client shows for the slot: the alias in ranked, else the name.
  // a match can come before its name does, so an empty name falls through.
  const title = player.alias || identity?.gameName || (player.isSelf ? "You" : "Teammate");
  const champion = championName(champions, player.championId);
  const picked = player.championId > 0;
  const ring: RingTone = player.isSelf ? "self" : player.acting ? "acting" : "teammate";

  return (
    <li
      ref={entry}
      style={style}
      className="fade-in-slow"
    >
      <article
        aria-label={title}
        className={cn("row", player.isSelf && "is-self")}
        data-open={open || undefined}
        data-acting={player.acting || undefined}
        data-expandable={expandable || undefined}
        // the triangle is the real control; the row is a bigger target for
        // the mouse.
        onClick={expandable ? onToggle : undefined}
      >
        <ChampionIcon
          championId={player.championId}
          champions={champions}
          size="portrait"
          ring={ring}
          position={player.position}
        />

        <div className="row-text">
          <div className="row-line row-line--role">
            <span className="row-role type-role">
              {player.position ? positionLabel(player.position) : null}
            </span>
            {stats ? (
              <RankBadge rank={stats.rank} unranked={stats.unranked} />
            ) : historyPending ? (
              <span aria-hidden className="skeleton h-3 w-20" />
            ) : null}
          </div>

          <div className="row-line row-line--champ">
            <p className="row-champ type-h5">
              {/* the champion table may still be loading, or not know a brand
                  new champion yet. */}
              {champion ?? (picked ? "Unknown champion" : "No pick yet")}
              {picked && stats && stats.championGames !== null ? (
                <span className="games">· {plural(stats.championGames, "game")}</span>
              ) : null}
            </p>
            <Form stats={stats} pending={historyPending} />
          </div>

          <div className="row-line row-line--name">
            <p className="row-name type-label">
              <Identity player={player} />
            </p>
          </div>

          <FlagLine
            player={player}
            pending={historyPending}
            expandable={expandable}
            expanded={open}
            detailId={detailId}
            onToggle={onToggle}
            title={title}
          />
        </div>
      </article>

      {open ? (
        <PlayerDetail
          id={detailId}
          player={player}
          stats={stats}
          champions={champions}
          scoutLabel={scoutLabel}
        />
      ) : null}
    </li>
  );
}

// line three: who the row is. your own riot id on your row; behind an alias,
// the alias and the matched riot id with how sure the match is; a name league
// shows (normal games, your party) on its own.
function Identity({ player }: { player: PlayerView }) {
  const { identity } = player;

  if (player.isSelf) {
    return (
      <>
        {identity && identity.gameName ? <RiotId identity={identity} /> : "You"}
        <span className="sr-only"> (you)</span>
      </>
    );
  }
  if (!player.alias) {
    return identity && identity.gameName ? (
      <>
        <RiotId identity={identity} />
        <MatchBadge identity={identity} />
      </>
    ) : null;
  }
  if (!identity) {
    return (
      <>
        {player.alias}
        <span className="sep">·</span>
        <span className="unnamed">{player.matching ? "Matching name…" : "Name hidden"}</span>
        {player.matching ? <Loader /> : null}
      </>
    );
  }
  return (
    <>
      {player.alias}
      <span className="sep">·</span>
      <RiotId identity={identity} />
      <MatchBadge identity={identity} />
    </>
  );
}

function RiotId({ identity }: { identity: IdentityView }) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <span className="match">
          {identity.gameName}
          {identity.tagLine ? <span className="tagline">#{identity.tagLine}</span> : null}
        </span>
      </TooltipTrigger>
      <TooltipContent>{riotId(identity)}</TooltipContent>
    </Tooltip>
  );
}

// a guess wears a "likely" tag toned by confidence; an exact match a small
// teal check. both explain themselves on hover.
function MatchBadge({ identity }: { identity: IdentityView }) {
  if (identity.kind === "own") {
    return null;
  }
  if (identity.kind === "guess") {
    const hint = `Best guess from recent games, ${identity.confidence} confidence`;
    return (
      <Tooltip>
        <TooltipTrigger asChild>
          <Tag tone={GUESS_TONES[identity.confidence]}>
            Likely
            <span className="sr-only">, {hint}</span>
          </Tag>
        </TooltipTrigger>
        <TooltipContent>{hint}</TooltipContent>
      </Tooltip>
    );
  }
  const hint = MATCH_HINTS[identity.kind];
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <span>
          <ExactGlyph />
          <span className="sr-only">{hint}</span>
        </span>
      </TooltipTrigger>
      <TooltipContent>{hint}</TooltipContent>
    </Tooltip>
  );
}

// line two, right: the last ten results and the kda over recent games.
function Form({ stats, pending }: { stats: StatsView | null; pending: boolean }) {
  if (pending && !stats?.historyLoaded) {
    return (
      <span aria-hidden className="flex items-center gap-2">
        <span className="skeleton h-1.5 w-19" />
        <span className="skeleton h-3 w-12" />
      </span>
    );
  }
  if (!stats) {
    return null;
  }
  if (!stats.historyLoaded) {
    return <span className="type-body">No match history</span>;
  }
  if (stats.recent.length === 0) {
    return <span className="type-body">No recent games</span>;
  }
  return (
    <span className="flex items-center gap-2">
      <FormDots games={stats.recent} />
      <span className="kda">
        {stats.kda !== null ? (
          <span className={stats.kda >= KDA_HIGH ? "high" : stats.kda < KDA_LOW ? "low" : undefined}>
            {formatKda(stats.kda)}
          </span>
        ) : (
          "–"
        )}
        <span className="label">KDA</span>
      </span>
    </span>
  );
}

// line four: the flag chips, "looks fine", or for your own row the account
// level and season, since you are never flagged. the details toggle sits at
// its end.
function FlagLine({
  player,
  pending,
  expandable,
  expanded,
  detailId,
  onToggle,
  title,
}: {
  player: PlayerView;
  pending: boolean;
  expandable: boolean;
  expanded: boolean;
  detailId: string;
  onToggle: () => void;
  title: string;
}) {
  const { stats, flags } = player;

  let content;
  if (pending && !player.isSelf) {
    content = (
      <span aria-hidden className="skeleton h-5.5 w-28 flex-none" style={{ borderRadius: 16 }} />
    );
  } else if (!stats) {
    // nothing known about this player (their name stayed hidden).
    content = <span />;
  } else if (player.isSelf) {
    content = <span className="row-quiet type-body">{selfSummary(stats)}</span>;
  } else if (flags.length === 0) {
    content = (
      <span className="row-fine type-body">
        <FineGlyph aria-hidden />
        Looks fine
      </span>
    );
  } else {
    content = <Flags flags={flags} />;
  }

  return (
    <div className="row-line row-line--flags">
      {content}
      {expandable ? (
        <button
          type="button"
          className="row-toggle"
          aria-label={expanded ? `Hide details for ${title}` : `Show details for ${title}`}
          aria-expanded={expanded}
          aria-controls={expanded ? detailId : undefined}
          onClick={(event) => {
            // the row toggles on click too; one click must not toggle twice.
            event.stopPropagation();
            onToggle();
          }}
        >
          <i aria-hidden />
        </button>
      ) : null}
    </div>
  );
}

// the flag chips, strongest first, on one line that keeps its height: the
// chips that fit, then a "+N" chip that names the rest on hover. screen
// readers hear every flag.
function Flags({ flags }: { flags: Flag[] }) {
  const list = useRef<HTMLUListElement>(null);
  const ruler = useRef<HTMLUListElement>(null);
  const shown = useFit(list, ruler, flags.map((flag) => flag.kind).join()) ?? flags.length;
  const rest = flags.slice(shown);

  return (
    <>
      <ul ref={list} aria-label="Flags" className="row-flags">
        {flags.map((flag, index) => (
          <li key={flag.kind} className={index < shown ? undefined : "sr-only"}>
            <FlagChip flag={flag} />
          </li>
        ))}
        {rest.length > 0 ? (
          <li aria-hidden className="row-more">
            <Tooltip>
              <TooltipTrigger asChild>
                <Chip tone="neutral">+{rest.length}</Chip>
              </TooltipTrigger>
              <TooltipContent>
                {rest.map((flag) => (
                  <span key={flag.kind} className="block">
                    {flagText(flag.label)}
                  </span>
                ))}
              </TooltipContent>
            </Tooltip>
          </li>
        ) : null}
      </ul>
      {/* every chip at its full width, and the widest "+N", to fit by. */}
      <ul ref={ruler} aria-hidden className="row-flags-ruler">
        {flags.map((flag) => (
          <li key={flag.kind}>
            <FlagChip flag={flag} />
          </li>
        ))}
        <li>
          <Chip tone="neutral">+{Math.max(flags.length - 1, 1)}</Chip>
        </li>
      </ul>
    </>
  );
}

function FlagChip({ flag }: { flag: Flag }) {
  const text = flagText(flag.label);
  return (
    <Chip tone={FLAG_TONES[flag.severity]} warn={flag.severity !== "info"} title={text}>
      {text}
    </Chip>
  );
}

// your own fourth line: level and ranked season.
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
