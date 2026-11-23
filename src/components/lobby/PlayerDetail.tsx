import { useId, type CSSProperties, type ReactNode } from "react";

import { LeagueArt } from "@/components/art/LeagueArt";
import { PositionGlyph } from "@/components/art/PositionGlyph";
import { ChampionIcon, championName } from "@/components/lobby/ChampionIcon";
import { Button } from "@/components/ui/button";
import { Raised } from "@/components/ui/frame";
import { useMinuteClock } from "@/hooks/useMinuteClock";
import { roleFile } from "@/lib/art";
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

// the five positions, in map order, for the role bars.
const POSITIONS: readonly Position[] = ["top", "jungle", "middle", "bottom", "utility"];

// the open part of a player row: a raised card pointing up at the portrait,
// with the account level, the ranked season and experience on the slot's
// champion, the roles they play, their last ten games and a link to their
// profile on the scout site. it is taller than the room under the pinned
// band, so opening a row scrolls it up under the band (see PlayerRow).
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
    <Raised id={id} className="details fade-in">
      <svg className="details-caret" viewBox="0 0 22 11" aria-hidden focusable={false}>
        <path d="M0 11 11 0 22 11Z" fill="var(--surface)" />
        <path d="M1 11 11 1 21 11" fill="none" stroke="#785a28" strokeWidth="2" />
      </svg>

      <dl className="details-section facts">
        <Fact label="Level">{stats.level ?? "Unknown"}</Fact>
        <span className="rule--v" aria-hidden />
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
              <span className="text-text-secondary"> · {formatPercent(rank.wins / ranked)}</span>
            </>
          ) : (
            "No games"
          )}
        </Fact>
        <span className="rule--v" aria-hidden />
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
        <div className="details-section">
          <Button
            variant="thin"
            external
            onClick={() => {
              if (player.scoutUrl) {
                bridge.openExternal(player.scoutUrl).catch(() => {});
              }
            }}
          >
            View on {scoutLabel}
          </Button>
        </div>
      ) : null}
    </Raised>
  );
}

function Fact({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div>
      <dt className="type-micro">{label}</dt>
      <dd className="type-value tabular-nums">{children}</dd>
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

// how their recent games split across the five roles, one bar each. the role
// this slot asks for is lit, so an autofilled teammate's short bar there
// stands out.
function Roles({ roles, slot }: { roles: RoleShare[]; slot: Position | null }) {
  const id = useId();
  const share = (position: Position) =>
    roles.find((role) => role.position === position)?.share ?? 0;
  const known = roles.some((role) => role.share > 0);
  return (
    <section aria-labelledby={id} className="details-section">
      <h3 id={id} className="details-label type-micro">
        Roles in recent games
      </h3>
      {known ? (
        <ul className="roles">
          {POSITIONS.map((position) => {
            const value = share(position);
            return (
              <li key={position} className={cn("role", position === slot && "is-slot")}>
                <span className="role-track" aria-hidden>
                  <span className="role-bar" style={{ "--share": value } as CSSProperties} />
                </span>
                <LeagueArt
                  file={roleFile(position)}
                  className="role-icon"
                  fallback={<PositionGlyph position={position} className="role-icon" />}
                />
                <span className="role-pct type-tiny">{formatPercent(value)}</span>
                <span className="sr-only">
                  {positionLabel(position)}
                  {position === slot ? ", this slot" : ""}
                </span>
              </li>
            );
          })}
        </ul>
      ) : (
        <p className="type-body">No role data yet.</p>
      )}
    </section>
  );
}

// the last games in two columns, read like the form dots: left to right,
// newest first. each shows the result, the champion's picture, k/d/a and how
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
    <section aria-labelledby={id} className="details-section">
      <h3 id={id} className="details-label type-micro">
        {games.length > 0 ? `Last ${plural(games.length, "game")}` : "Recent games"}
      </h3>
      {games.length > 0 ? (
        <ol className="games-list">
          {games.map((game, index) => {
            const champion = championName(champions, game.championId) ?? "an unknown champion";
            const length = formatMinutes(game.durationS);
            const ago = timeAgo(game.startedAtMs, now);
            return (
              <li
                key={`${game.startedAtMs}-${index}`}
                title={`${capitalize(champion)}, ${length} game`}
                className="game type-body"
              >
                <span aria-hidden className={game.win ? "w" : "l"}>
                  {game.win ? "W" : "L"}
                </span>
                <ChampionIcon championId={game.championId} champions={champions} size="square" />
                <span aria-hidden className="kda-line tabular-nums">
                  {game.kills} / {game.deaths} / {game.assists}
                </span>
                <span aria-hidden className="ago tabular-nums">
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
        <p className="type-body">
          {loaded ? "No Summoner's Rift games lately." : "No match history to show."}
        </p>
      )}
    </section>
  );
}
