// sample data for the browser preview (npm run dev): a five player lobby in a
// few states, the champion table, settings and an update notice. the riot ids
// are made up; the champion ids, image ids and the data dragon version are
// real, so the icons load. shapes follow src/types exactly, and labels are
// written the way the rust side words them.

import type { PreviewScreen } from "@/lib/bridge";
import type {
  Acting,
  ChampionData,
  ConnectionStatus,
  Flag,
  GameView,
  LobbyView,
  MatchKind,
  NameView,
  PlayerView,
  Position,
  RankView,
  RoleShare,
  ScoutProvider,
  Settings,
  Stage,
  StatsView,
  Summary,
  UpdateInfo,
} from "@/types";

export const DDRAGON_VERSION = "16.23.1";

// [numeric key, data dragon image id, display name], for every champion the
// fixtures mention.
const CHAMPION_ROWS: [number, string, string][] = [
  [20, "Nunu", "Nunu & Willump"],
  [24, "Jax", "Jax"],
  [51, "Caitlyn", "Caitlyn"],
  [58, "Renekton", "Renekton"],
  [61, "Orianna", "Orianna"],
  [64, "LeeSin", "Lee Sin"],
  [81, "Ezreal", "Ezreal"],
  [89, "Leona", "Leona"],
  [99, "Lux", "Lux"],
  [103, "Ahri", "Ahri"],
  [111, "Nautilus", "Nautilus"],
  [117, "Lulu", "Lulu"],
  [134, "Syndra", "Syndra"],
  [145, "Kaisa", "Kai'Sa"],
  [222, "Jinx", "Jinx"],
  [266, "Aatrox", "Aatrox"],
  [412, "Thresh", "Thresh"],
  [498, "Xayah", "Xayah"],
  [517, "Sylas", "Sylas"],
  [799, "Ambessa", "Ambessa"],
  [800, "Mel", "Mel"],
  [897, "KSante", "K'Sante"],
  [910, "Hwei", "Hwei"],
];

export const CHAMPIONS: ChampionData = {
  version: DDRAGON_VERSION,
  champions: Object.fromEntries(
    CHAMPION_ROWS.map(([key, id, name]) => [String(key), { id, name }]),
  ),
};

export const SETTINGS: Settings = {
  scoutProvider: "opgg",
  autoOpen: false,
  revealRanked: true,
  launchAtLogin: false,
  dodgeLeadMs: 1500,
  welcomed: true,
};

export const UPDATE: UpdateInfo = {
  current: "0.2.0",
  latest: "0.2.1",
  available: true,
  url: "https://github.com/ibrahimyurdan/expose/releases/latest",
};

// what the header would report on each preview screen.
export function statusFor(screen: PreviewScreen): ConnectionStatus {
  switch (screen) {
    case "waiting":
    case "welcome":
      return "waiting";
    case "lobby":
      return "champSelect";
    case "idle":
    case "settings":
    case "gallery":
      return "connected";
  }
}

// ranked: names matched through the scramble key, one teammate's stats still
// loading. guess: the key could not be recovered, so four best guesses.
// resolving: the first moments of a ranked lobby, names known but not yet tied
// to slots. hidden: a ranked lobby without the riot client, so no names at all.
// normal: a draft game where nothing is hidden. pick: ranked again, with your
// jungler on the clock to pick. long: ranked with the longest names and
// labels, and the widest match marker, to check rows at the smallest window.
// aram: an aram lobby, with no positions and no turns.
export type LobbyVariant =
  | "ranked"
  | "guess"
  | "resolving"
  | "hidden"
  | "normal"
  | "pick"
  | "long"
  | "aram";

export const LOBBY_VARIANTS: readonly LobbyVariant[] = [
  "ranked",
  "guess",
  "resolving",
  "hidden",
  "normal",
  "pick",
  "long",
  "aram",
];

export interface LobbyOptions {
  provider?: ScoutProvider;
  // players (by key) whose stats are still loading. defaults per variant.
  pending?: readonly string[];
  leadMs?: number;
  now?: number;
}

// one recent game: champion, result, kills, deaths, assists, hours ago and
// length in minutes.
type GameRow = [number, "W" | "L", number, number, number, number, number];

interface FixturePlayer {
  key: string;
  cellId: number;
  isSelf: boolean;
  alias: string;
  position: Position;
  isAutofilled: boolean;
  championId: number;
  spells: [number, number];
  name: NameView;
  level: number;
  rank: RankView | null;
  games: GameRow[];
  winRate: number;
  championGames: number;
  championPoints: number;
  roles: RoleShare[];
  flags: Flag[];
}

// summoner spell ids: flash 4, ignite 14, teleport 12, smite 11, heal 7,
// exhaust 3.
const TEAM: FixturePlayer[] = [
  {
    key: "3f6c1d2e-8a4b-4f0e-9c71-5d2a8e6b4f13",
    cellId: 0,
    isSelf: false,
    alias: "Gromp",
    position: "top",
    isAutofilled: false,
    championId: 266,
    spells: [4, 12],
    name: { gameName: "Driftwood Ranger", tagLine: "NA1" },
    level: 214,
    rank: { tier: "GOLD", division: "II", lp: 54, wins: 61, losses: 55 },
    games: [
      [266, "W", 7, 3, 6, 1.2, 31],
      [266, "W", 5, 4, 9, 2, 28],
      [58, "L", 3, 5, 4, 3.1, 34],
      [266, "W", 9, 2, 5, 20, 26],
      [24, "L", 4, 6, 3, 21, 33],
      [266, "W", 6, 3, 8, 22, 29],
      [799, "W", 8, 4, 4, 46, 30],
      [266, "L", 2, 5, 6, 47, 36],
      [58, "W", 10, 3, 7, 49, 27],
      [897, "W", 4, 2, 11, 70, 32],
    ],
    winRate: 0.6,
    championGames: 7,
    championPoints: 186_400,
    roles: [
      { position: "top", share: 0.85 },
      { position: "middle", share: 0.1 },
      { position: "jungle", share: 0.05 },
    ],
    flags: [],
  },
  {
    key: "b81e5a07-2c9d-4e36-a1f4-7e0c3b9d2a58",
    cellId: 1,
    isSelf: false,
    alias: "Krug",
    position: "jungle",
    isAutofilled: true,
    championId: 64,
    spells: [11, 4],
    name: { gameName: "kumo", tagLine: "4421" },
    level: 167,
    rank: { tier: "SILVER", division: "I", lp: 12, wins: 38, losses: 44 },
    games: [
      [61, "L", 2, 7, 3, 0.6, 29],
      [134, "L", 4, 8, 2, 1.3, 33],
      [61, "L", 1, 6, 5, 2.1, 27],
      [103, "L", 3, 9, 4, 2.9, 35],
      [61, "W", 8, 3, 6, 26, 30],
      [134, "L", 2, 5, 7, 27, 31],
      [61, "W", 6, 4, 9, 28, 29],
      [99, "L", 3, 7, 6, 50, 32],
      [61, "L", 4, 6, 2, 51, 28],
      [134, "W", 7, 2, 8, 52, 30],
    ],
    winRate: 0.38,
    championGames: 0,
    championPoints: 24_100,
    roles: [
      { position: "middle", share: 0.69 },
      { position: "top", share: 0.19 },
      { position: "utility", share: 0.12 },
    ],
    flags: [
      { kind: "autofilled", severity: "red", label: "autofilled, mains mid" },
      { kind: "lossStreak", severity: "red", label: "lost last 4" },
      { kind: "lowWinRate", severity: "amber", label: "38% win rate, last 16" },
    ],
  },
  {
    key: "e2a94c6b-71d3-4b85-8f2e-c45a1d07b9e6",
    cellId: 2,
    isSelf: true,
    alias: "Murk Wolf",
    position: "middle",
    isAutofilled: false,
    championId: 103,
    spells: [4, 14],
    name: { gameName: "Lantern Keeper", tagLine: "NA1" },
    level: 302,
    rank: { tier: "GOLD", division: "III", lp: 80, wins: 54, losses: 47 },
    games: [
      [103, "W", 8, 2, 7, 1, 27],
      [103, "L", 4, 5, 6, 2, 33],
      [517, "W", 6, 3, 9, 3, 30],
      [103, "W", 11, 4, 5, 22, 29],
      [910, "W", 5, 2, 12, 23, 31],
      [103, "L", 3, 6, 4, 24, 35],
      [103, "W", 7, 1, 8, 45, 26],
      [800, "L", 2, 4, 5, 46, 32],
      [103, "W", 9, 3, 6, 47, 28],
      [103, "W", 6, 2, 10, 70, 30],
    ],
    winRate: 0.65,
    championGames: 9,
    championPoints: 312_500,
    roles: [
      { position: "middle", share: 0.9 },
      { position: "top", share: 0.1 },
    ],
    flags: [],
  },
  {
    key: "7c03f8d1-5e2a-4a9b-b6c4-19d8e2f05a37",
    cellId: 3,
    isSelf: false,
    alias: "Raptor",
    position: "bottom",
    isAutofilled: false,
    championId: 222,
    spells: [7, 4],
    name: { gameName: "Paper Plane", tagLine: "8812" },
    level: 96,
    rank: { tier: "GOLD", division: "IV", lp: 0, wins: 20, losses: 25 },
    games: [
      [145, "L", 3, 8, 4, 1.5, 31],
      [51, "W", 6, 5, 3, 2.4, 29],
      [81, "L", 2, 7, 5, 3.3, 33],
      [145, "L", 4, 9, 2, 24, 34],
      [51, "L", 3, 6, 4, 25, 30],
      [81, "W", 7, 4, 6, 26, 28],
      [498, "L", 1, 8, 5, 48, 32],
      [145, "W", 5, 5, 4, 49, 30],
      [51, "L", 2, 7, 3, 50, 31],
      [81, "L", 4, 6, 2, 72, 29],
    ],
    winRate: 0.45,
    championGames: 0,
    championPoints: 0,
    roles: [
      { position: "bottom", share: 0.9 },
      { position: "utility", share: 0.1 },
    ],
    flags: [
      { kind: "firstTime", severity: "red", label: "first time on Jinx" },
      { kind: "badKda", severity: "amber", label: "1.2 kda, last 10" },
    ],
  },
  {
    key: "a5d27e94-0b6f-4c13-9e8a-62f1c7b3d0e4",
    cellId: 4,
    isSelf: false,
    alias: "Scuttle Crab",
    position: "utility",
    isAutofilled: false,
    championId: 412,
    spells: [4, 3],
    name: { gameName: "Saltmarsh", tagLine: "HOOK" },
    level: 41,
    rank: { tier: "PLATINUM", division: "IV", lp: 22, wins: 15, losses: 9 },
    games: [
      [412, "L", 1, 6, 9, 0.4, 30],
      [412, "W", 2, 3, 17, 0.9, 28],
      [111, "L", 0, 7, 8, 1.7, 33],
      [412, "L", 1, 5, 6, 2.6, 29],
      [89, "L", 0, 8, 11, 3.5, 31],
      [412, "W", 3, 2, 14, 4.4, 26],
      [412, "W", 1, 4, 19, 26, 30],
      [117, "W", 0, 3, 21, 27, 28],
      [412, "W", 2, 2, 16, 49, 31],
      [412, "W", 1, 5, 13, 50, 27],
    ],
    winRate: 0.6,
    championGames: 7,
    championPoints: 96_700,
    roles: [
      { position: "utility", share: 0.95 },
      { position: "bottom", share: 0.05 },
    ],
    flags: [
      { kind: "freshAccount", severity: "amber", label: "level 41 in platinum" },
      { kind: "onTilt", severity: "amber", label: "6 games in 6 hours" },
    ],
  },
];

const HOUR_MS = 3_600_000;

function recentGames(rows: GameRow[], now: number): GameView[] {
  return rows.map(([championId, result, kills, deaths, assists, hoursAgo, minutes]) => ({
    championId,
    win: result === "W",
    kills,
    deaths,
    assists,
    startedAtMs: Math.round(now - hoursAgo * HOUR_MS),
    durationS: minutes * 60,
  }));
}

function statsFor(player: FixturePlayer, now: number): StatsView {
  const recent = recentGames(player.games, now);
  const total = (pick: (game: GameView) => number) =>
    recent.reduce((sum, game) => sum + pick(game), 0);
  const kills = total((game) => game.kills);
  const deaths = total((game) => game.deaths);
  const assists = total((game) => game.assists);
  return {
    level: player.level,
    rank: player.rank,
    unranked: player.rank === null,
    historyLoaded: true,
    recent,
    kda: (kills + assists) / Math.max(1, deaths),
    winRate: player.winRate,
    championGames: player.championGames,
    championPoints: player.championPoints,
    roles: player.roles,
  };
}

// the single-summoner profile link the backend builds for the chosen site.
function profileUrl(provider: ScoutProvider, name: NameView): string {
  const riotId = `${encodeURIComponent(name.gameName)}-${encodeURIComponent(name.tagLine)}`;
  return provider === "ugg"
    ? `https://u.gg/lol/profile/na1/${riotId}/overview`
    : `https://www.op.gg/summoners/na/${riotId}`;
}

function plural(count: number, word: string): string {
  return `${count} ${word}${count === 1 ? "" : "s"}`;
}

// the same counting rule as lobby/flags.rs: teammates only, red flags first.
function summarize(players: PlayerView[]): Summary {
  const teammates = players.filter((player) => !player.isSelf);
  const reds = teammates.map(
    (player) => player.flags.filter((flag) => flag.severity === "red").length,
  );
  const redFlags = reds.reduce((sum, count) => sum + count, 0);
  const redPlayers = reds.filter((count) => count > 0).length;
  const warnings = teammates
    .flatMap((player) => player.flags)
    .filter((flag) => flag.severity === "amber").length;
  const label =
    redFlags > 0
      ? `${plural(redFlags, "red flag")} on ${plural(redPlayers, "teammate")}`
      : warnings > 0
        ? plural(warnings, "warning")
        : "no red flags";
  return { redFlags, redPlayers, warnings, label };
}

const DEFAULT_PENDING: Record<LobbyVariant, readonly string[]> = {
  ranked: [TEAM[4]?.key ?? ""],
  guess: [],
  resolving: [],
  hidden: [],
  normal: [],
  pick: [],
  long: [],
  aram: [],
};

interface Timer {
  phase: string;
  timeLeftMs: number;
  totalMs: number;
  stage: Stage;
  acting: Acting;
  // the cells of your team on the clock.
  actingCells: readonly number[];
}

// ranked: simultaneous bans, everyone banning. guess: the enemy picks.
// resolving: planning. hidden: your bot laner picks. normal: lock-in. pick:
// your jungler picks.
const TIMERS: Record<LobbyVariant, Timer> = {
  ranked: {
    phase: "BAN_PICK",
    timeLeftMs: 27_000,
    totalMs: 30_000,
    stage: "ban",
    acting: "both",
    actingCells: [0, 1, 2, 3, 4],
  },
  guess: {
    phase: "BAN_PICK",
    timeLeftMs: 21_000,
    totalMs: 30_000,
    stage: "pick",
    acting: "enemy",
    actingCells: [],
  },
  resolving: {
    phase: "PLANNING",
    timeLeftMs: 24_000,
    totalMs: 30_000,
    stage: "planning",
    acting: "none",
    actingCells: [],
  },
  hidden: {
    phase: "BAN_PICK",
    timeLeftMs: 25_000,
    totalMs: 30_000,
    stage: "pick",
    acting: "ally",
    actingCells: [3],
  },
  normal: {
    phase: "FINALIZATION",
    timeLeftMs: 9_000,
    totalMs: 30_000,
    stage: "finalization",
    acting: "none",
    actingCells: [],
  },
  pick: {
    phase: "BAN_PICK",
    timeLeftMs: 22_000,
    totalMs: 30_000,
    stage: "pick",
    acting: "ally",
    actingCells: [1],
  },
  long: {
    phase: "BAN_PICK",
    timeLeftMs: 19_000,
    totalMs: 30_000,
    stage: "pick",
    acting: "ally",
    actingCells: [1],
  },
  aram: {
    phase: "BAN_PICK",
    timeLeftMs: 41_000,
    totalMs: 60_000,
    stage: "unknown",
    acting: "none",
    actingCells: [],
  },
};

// the long lobby's jungler: a 16-character name with a 5-character tag,
// grandmaster, three red flags, on nunu & willump.
const LONG_JUNGLER = {
  name: { gameName: "Wwwwwwwwwwwwwwww", tagLine: "WWWWW" },
  championId: 20,
  rank: { tier: "GRANDMASTER", division: "I", lp: 1234, wins: 212, losses: 187 },
  flags: [
    { kind: "autofilled", severity: "red", label: "autofilled, mains mid" },
    { kind: "lossStreak", severity: "red", label: "lost last 4" },
    { kind: "firstTime", severity: "red", label: "first time on nunu & willump" },
  ],
} as const;

function matchKind(player: FixturePlayer, variant: LobbyVariant): MatchKind {
  if (player.isSelf) {
    return "own";
  }
  switch (variant) {
    // long: a guess, since "likely" is the widest match marker.
    case "guess":
    case "long":
      return "guess";
    case "normal":
      return "visible";
    default:
      return "key";
  }
}

export function lobbyFixture(variant: LobbyVariant, options: LobbyOptions = {}): LobbyView {
  const now = options.now ?? Date.now();
  const provider = options.provider ?? "opgg";
  const pending = options.pending ?? DEFAULT_PENDING[variant];
  const anonymous = variant !== "normal" && variant !== "aram";
  const { actingCells, ...timer } = TIMERS[variant];

  const players = TEAM.map((base): PlayerView => {
    const player: FixturePlayer =
      variant === "long" && base.cellId === 1
        ? {
            ...base,
            name: { ...LONG_JUNGLER.name },
            championId: LONG_JUNGLER.championId,
            rank: { ...LONG_JUNGLER.rank },
            flags: LONG_JUNGLER.flags.map((flag) => ({ ...flag })),
          }
        : base;
    const resolved = player.isSelf || (variant !== "resolving" && variant !== "hidden");
    const kind = matchKind(player, variant);
    const loading = resolved && pending.includes(player.key);
    const stats = resolved && !loading ? statsFor(player, now) : null;
    return {
      key: player.key,
      cellId: player.cellId,
      isSelf: player.isSelf,
      alias: anonymous && !player.isSelf ? player.alias : null,
      // aram has no positions.
      position: variant === "aram" ? null : player.position,
      isAutofilled: variant !== "aram" && player.isAutofilled,
      // while declaring, the support has not hovered a champion yet.
      championId: variant === "resolving" && player.cellId === 4 ? 0 : player.championId,
      spells: player.spells,
      identity: resolved
        ? { ...player.name, kind, confidence: kind === "guess" ? "medium" : "high" }
        : null,
      // resolving: the chat answered and the names are being placed. hidden:
      // the riot client never answered, so there is nothing left to try.
      matching: !resolved && variant === "resolving",
      stats,
      statsPending: loading,
      // an aram slot has no role, so nobody is autofilled.
      flags:
        stats && !player.isSelf
          ? player.flags.filter((flag) => variant !== "aram" || flag.kind !== "autofilled")
          : [],
      scoutUrl: resolved ? profileUrl(provider, player.name) : null,
      acting: actingCells.includes(player.cellId),
    };
  });

  // the riot client lists names in its own order, not the slot order.
  const unmatched =
    variant === "resolving"
      ? [3, 0, 4, 1].flatMap((index) => {
          const player = TEAM[index];
          return player ? [player.name] : [];
        })
      : [];

  return {
    gameId: variant === "normal" ? 5_210_351_266 : 5_210_337_904,
    queueId: variant === "normal" ? 400 : variant === "aram" ? 450 : 420,
    ...timer,
    anonymous,
    players,
    unmatched,
    summary: summarize(players),
    dodge: { armed: false, leadMs: options.leadMs ?? SETTINGS.dodgeLeadMs },
  };
}

// every teammate's key, for a lobby whose stats have not arrived yet.
export const TEAMMATE_KEYS: readonly string[] = TEAM.filter((player) => !player.isSelf).map(
  (player) => player.key,
);
