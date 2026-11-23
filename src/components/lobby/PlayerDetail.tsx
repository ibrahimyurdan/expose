import { useId, type CSSProperties, type ReactNode } from "react";

import { LeagueArt } from "@/components/art/LeagueArt";
import { PositionGlyph } from "@/components/art/PositionGlyph";
import { RankCrest } from "@/components/art/RankCrest";
import { ChampionIcon, championName } from "@/components/lobby/ChampionIcon";
import { Button } from "@/components/ui/button";
import { Raised } from "@/components/ui/frame";
import { useMinuteClock } from "@/hooks/useMinuteClock";
import { crestFile, emblemFile, roleFile } from "@/lib/art";
import { bridge } from "@/lib/bridge";
import {
  capitalize,
  formatMinutes,
  formatPercent,
  formatPoints,
  isApexTier,
  plural,
  positionLabel,
  tierName,
  timeAgo,
} from "@/lib/format";
import { cn } from "@/lib/utils";
import type { ChampionData, GameView, PlayerView, Position, RoleShare, StatsView } from "@/types";

// the five positions, in map order, for the role bars.
const POSITIONS: readonly Position[] = ["top", "jungle", "middle", "bottom", "utility"];

// the open part of a player row: a raised card pointing up at the portrait,
// with the solo/duo rank under the tier's emblem, the account level and
// experience on the slot's champion, the roles they play, their last ten
// games and a link to their profile on the scout site. it is taller than the room under the pinned
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

  return (
    <Raised id={id} className="details fade-in">
      <svg className="details-caret" viewBox="0 0 22 11" aria-hidden focusable={false}>
        <path d="M0 11 11 0 22 11Z" fill="var(--surface)" />
        <path d="M1 11 11 1 21 11" fill="none" stroke="#785a28" strokeWidth="2" />
      </svg>

      <RankHead stats={stats} />

      <dl className="details-section facts">
        <Fact label="Level">{stats.level ?? "Unknown"}</Fact>
        <span className="rule--v" aria-hidden />
        <Fact label={champion ? `On ${champion}` : "Champion"}>
          {player.championId <= 0
            ? "No pick yet"
            : stats.championGames !== null
              ? plural(stats.championGames, "game")
              : "Unknown"}
        </Fact>
        <span className="rule--v" aria-hidden />
        <Fact label="Mastery">
          {player.championId <= 0
            ? "–"
            : stats.championPoints !== null
              ? `${formatPoints(stats.championPoints)} pts`
              : "Unknown"}
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

// the solo/duo rank: the tier's full emblem (our crest until it loads), the
// rank and lp, then the season's record. unranked shows the unranked crest; a
// rank that could not be loaded says so.
function RankHead({ stats }: { stats: StatsView }) {
  const { rank } = stats;
  let art: ReactNode = null;
  let title: ReactNode;
  let record = "Ranked solo/duo";
  if (rank) {
    const tier = rank.tier.toUpperCase();
    const division = isApexTier(tier) ? "" : rank.division;
    const games = rank.wins + rank.losses;
    art = (
      <LeagueArt
        file={emblemFile(tier)}
        className="rank-emblem-art"
        fallback={<RankCrest tier={tier} className="rank-emblem-crest" />}
      />
    );
    title = (
      <>
        {tierName(tier)}
        {division ? ` ${division}` : null}
        <span className="lp"> · {rank.lp} LP</span>
      </>
    );
    record +=
      games > 0
        ? ` · ${rank.wins}W ${rank.losses}L · ${formatPercent(rank.wins / games)}`
        : " · No games";
  } else if (stats.unranked) {
    art = <LeagueArt file={crestFile("unranked")} className="rank-emblem-crest" fallback={null} />;
    title = "Unranked";
  } else {
    title = "Rank unknown";
  }
  return (
    <div className="details-section rank-head">
      {art ? (
        <span aria-hidden className="rank-emblem">
          {art}
        </span>
      ) : null}
      <div className="rank-head-text">
        <p className="rank-head-title type-h5">{title}</p>
        <p className="rank-head-record type-body tabular-nums">{record}</p>
      </div>
    </div>
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
