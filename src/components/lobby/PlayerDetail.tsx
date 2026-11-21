import { ExternalLink } from "lucide-react";
import { useId, type ReactNode } from "react";

import { ChampionIcon, championName } from "@/components/lobby/ChampionIcon";
import { Button } from "@/components/ui/button";
import { useMinuteClock } from "@/hooks/useMinuteClock";
import { bridge } from "@/lib/bridge";
import {
  capitalize,
  formatMinutes,
  formatPercent,
  formatPoints,
  plural,
  positionLabel,
  timeAgo,
} from "@/lib/format";
import { cn } from "@/lib/utils";
import type { ChampionData, GameView, PlayerView, Position, RoleShare, StatsView } from "@/types";

// the expanded part of a player row: account level, the ranked season and
// experience on the slot's champion, the roles they play, their last ten games
// and a link to their profile on the scout site. kept compact, so an open row
// still fits under the pinned band at the smallest window size.
export function PlayerDetail({
  id,
  player,
  stats,
  champions,
  scoutLabel,
}: {
  id: string;
  player: PlayerView;
  stats: StatsView;
  champions: ChampionData | null;
  // the scout site's name, for example "OP.GG".
  scoutLabel: string;
}) {
  const champion = championName(champions, player.championId);
  const rank = stats.rank;
  const ranked = rank ? rank.wins + rank.losses : 0;

  return (
    <div
      id={id}
      className="flex animate-in flex-col gap-4 border-t border-border p-3 duration-200 fade-in-0"
    >
      <dl className="grid grid-cols-3 gap-3">
        <Fact label="Level">{stats.level ?? "Unknown"}</Fact>
        <Fact label="Ranked this season">
          {!rank ? (
            stats.unranked ? (
              "Unranked"
            ) : (
              "Unknown"
            )
          ) : ranked > 0 ? (
            <>
              {rank.wins}W {rank.losses}L
              <span className="text-fg-secondary"> · {formatPercent(rank.wins / ranked)}</span>
            </>
          ) : (
            "No games"
          )}
        </Fact>
        <Fact label={champion ? `On ${champion}` : "Champion"}>
          {player.championId > 0 ? <ChampionExperience stats={stats} /> : "No pick yet"}
        </Fact>
      </dl>

      <Roles roles={stats.roles} slot={player.position} />

      <RecentGames
        games={stats.recent.slice(0, 10)}
        loaded={stats.historyLoaded}
        champions={champions}
      />

      {player.scoutUrl ? (
        <Button
          size="sm"
          className="self-start"
          onClick={() => {
            if (player.scoutUrl) {
              bridge.openExternal(player.scoutUrl).catch(() => {});
            }
          }}
        >
          View on {scoutLabel}
          <ExternalLink aria-hidden />
        </Button>
      ) : null}
    </div>
  );
}

function Fact({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="flex min-w-0 flex-col gap-0.5">
      <dt className="truncate text-xs text-fg-secondary">{label}</dt>
      <dd className="truncate text-sm text-fg tabular-nums">{children}</dd>
    </div>
  );
}

// recent games and mastery on the slot's champion, each only when known.
function ChampionExperience({ stats }: { stats: StatsView }) {
  const parts: string[] = [];
  if (stats.championGames !== null) {
    parts.push(plural(stats.championGames, "game"));
  }
  if (stats.championPoints !== null) {
    parts.push(`${formatPoints(stats.championPoints)} pts`);
  }
  return <>{parts.length > 0 ? parts.join(" · ") : "Unknown"}</>;
}

// how their recent games split across roles, as one bar. the role this slot
// asks for is drawn in the accent color, and listed at 0% when they never play
// it, so an autofill stands out.
function Roles({ roles, slot }: { roles: RoleShare[]; slot: Position | null }) {
  const id = useId();
  const shown = roles.filter((role) => role.share > 0);
  const legend =
    slot && !shown.some((role) => role.position === slot)
      ? [...shown, { position: slot, share: 0 }]
      : shown;
  return (
    <section aria-labelledby={id} className="flex flex-col gap-1.5">
      <h3 id={id} className="text-xs text-fg-secondary">
        Roles in recent games
      </h3>
      {shown.length > 0 ? (
        <>
          <div aria-hidden className="flex h-1.5 gap-0.5 overflow-hidden rounded-full">
            {shown.map((role) => (
              <span
                key={role.position}
                className={cn("h-full", role.position === slot ? "bg-accent" : "bg-border-strong")}
                style={{ flexGrow: role.share, flexBasis: 0 }}
              />
            ))}
          </div>
          <ul className="flex flex-wrap gap-x-3 text-sm">
            {legend.map((role) => (
              <li
                key={role.position}
                className={cn(
                  "flex items-center gap-1.5 tabular-nums",
                  role.position === slot ? "font-medium text-fg" : "text-fg-secondary",
                )}
              >
                <span
                  aria-hidden
                  className={cn(
                    "size-1.5 rounded-full",
                    role.position === slot ? "bg-accent" : "bg-border-strong",
                  )}
                />
                {positionLabel(role.position)} {formatPercent(role.share)}
                {role.position === slot ? <span className="sr-only">, this slot</span> : null}
              </li>
            ))}
          </ul>
        </>
      ) : (
        <p className="text-sm text-fg-secondary">No role data yet.</p>
      )}
    </section>
  );
}

// the last games in two columns, read like the form dots: left to right,
// newest first. each shows the result, the champion's portrait, k/d/a and how
// long ago; the champion's name and the game's length are on hover, and
// screen readers hear each game as one sentence.
function RecentGames({
  games,
  loaded,
  champions,
}: {
  games: GameView[];
  // a match history came back; without one there is nothing to list.
  loaded: boolean;
  champions: ChampionData | null;
}) {
  const id = useId();
  const now = useMinuteClock();
  return (
    <section aria-labelledby={id} className="flex flex-col gap-1">
      <h3 id={id} className="text-xs text-fg-secondary">
        {games.length > 0 ? `Last ${plural(games.length, "game")}` : "Recent games"}
      </h3>
      {games.length > 0 ? (
        <ol className="grid grid-cols-2 gap-x-6">
          {games.map((game, index) => {
            const champion = championName(champions, game.championId) ?? "an unknown champion";
            const length = formatMinutes(game.durationS);
            const ago = timeAgo(game.startedAtMs, now);
            return (
              <li
                key={`${game.startedAtMs}-${index}`}
                title={`${capitalize(champion)}, ${length} game`}
                className="flex h-6 min-w-0 items-center gap-2 text-sm"
              >
                <span
                  aria-hidden
                  className={cn("w-3 shrink-0 font-semibold", game.win ? "text-green" : "text-red")}
                >
                  {game.win ? "W" : "L"}
                </span>
                <ChampionIcon championId={game.championId} champions={champions} size="sm" />
                <span aria-hidden className="whitespace-nowrap text-fg tabular-nums">
                  {game.kills} / {game.deaths} / {game.assists}
                </span>
                <span
                  aria-hidden
                  className="ml-auto whitespace-nowrap text-fg-secondary tabular-nums"
                >
                  {ago}
                </span>
                <span className="sr-only">
                  {game.win ? "Won" : "Lost"} on {champion}, {game.kills} kills, {game.deaths}{" "}
                  deaths, {game.assists} assists, {length}, {ago}.
                </span>
              </li>
            );
          })}
        </ol>
      ) : (
        <p className="text-sm text-fg-secondary">
          {loaded ? "No Summoner's Rift games lately." : "No match history to show."}
        </p>
      )}
    </section>
  );
}
