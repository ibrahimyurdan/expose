// these mirror the serde payloads the rust backend sends. the lobby shapes are
// the shared contract with src-tauri/src/lobby/view.rs (camelCase fields,
// option fields arrive as null); keep the two in step.

export type ConnectionStatus = "waiting" | "connected" | "champSelect";

// ---- champ select lobby (event expose://lobby, command get_lobby) ----

export type Position = "top" | "jungle" | "middle" | "bottom" | "utility";

// how a teammate's riot id was tied to their slot. own, visible, key and
// elimination are exact; guess is a best guess from their recent games.
export type MatchKind = "own" | "visible" | "key" | "elimination" | "guess";

export type Confidence = "high" | "medium" | "low";

export type Severity = "red" | "amber" | "info";

export type FlagKind =
  | "autofilled"
  | "lossStreak"
  | "firstTime"
  | "newToChampion"
  | "lowWinRate"
  | "badKda"
  | "onTilt"
  | "freshAccount"
  | "noData";

// where champ select stands. unknown when the session does not say (a pick
// and ban phase with nobody on the clock, say).
export type Stage = "unknown" | "planning" | "ban" | "pick" | "finalization";

// which team is on a ban or pick turn right now.
export type Acting = "none" | "ally" | "enemy" | "both";

export interface LobbyView {
  gameId: number;
  queueId: number;
  // raw lcu timer phase: PLANNING, BAN_PICK, FINALIZATION and so on.
  phase: string;
  // time left in the phase when the backend built this view.
  timeLeftMs: number;
  // the phase's full length; 0 when unknown.
  totalMs: number;
  stage: Stage;
  acting: Acting;
  // true when riot hides teammate names (ranked solo/duo).
  anonymous: boolean;
  // your team in slot order, you included.
  players: PlayerView[];
  // riot ids from the riot client that are not tied to a slot yet.
  unmatched: NameView[];
  // teammates only, never you.
  summary: Summary;
  dodge: DodgeView;
}

export interface PlayerView {
  // stable per player for the whole lobby (cells change on pick-order swaps).
  key: string;
  cellId: number;
  isSelf: boolean;
  // the client's stand-in name (Gromp, Krug…) when names are hidden.
  alias: string | null;
  position: Position | null;
  isAutofilled: boolean;
  // the locked or hovered champion, 0 when there is none.
  championId: number;
  spells: [number, number];
  identity: IdentityView | null;
  // hidden, not matched yet, and the backend is still working on it (the
  // chat has not answered, or a guess waits for stats). false once there is
  // nothing left to try: the name stays hidden.
  matching: boolean;
  stats: StatsView | null;
  // stats are being fetched for this player.
  statsPending: boolean;
  // red first, then amber, then info.
  flags: Flag[];
  scoutUrl: string | null;
  // banning or picking right now.
  acting: boolean;
}

export interface IdentityView {
  gameName: string;
  tagLine: string;
  kind: MatchKind;
  // always high for the exact kinds.
  confidence: Confidence;
}

export interface NameView {
  gameName: string;
  tagLine: string;
}

// each piece is null when its own request failed or has no data.
export interface StatsView {
  level: number | null;
  // ranked solo/duo, null when unranked or when it could not be loaded.
  rank: RankView | null;
  // the ranked stats came back without a solo/duo rank. rank null with this
  // false means the rank is unknown.
  unranked: boolean;
  // a match history with games came back. false when it failed or was empty.
  historyLoaded: boolean;
  // the last summoner's rift games, newest first.
  recent: GameView[];
  // (kills + assists) / deaths over the recent games.
  kda: number | null;
  // share of recent games won, 0 to 1.
  winRate: number | null;
  // recent games on the slot's champion.
  championGames: number | null;
  // mastery points on the slot's champion.
  championPoints: number | null;
  // most played first, shares from 0 to 1.
  roles: RoleShare[];
}

export interface RankView {
  // IRON through CHALLENGER.
  tier: string;
  // I to IV, empty for master and above.
  division: string;
  lp: number;
  wins: number;
  losses: number;
}

export interface GameView {
  championId: number;
  win: boolean;
  kills: number;
  deaths: number;
  assists: number;
  startedAtMs: number;
  durationS: number;
}

export interface RoleShare {
  position: Position;
  share: number;
}

export interface Flag {
  kind: FlagKind;
  severity: Severity;
  // ready to show, for example "lost last 4".
  label: string;
}

export interface Summary {
  redFlags: number;
  redPlayers: number;
  warnings: number;
  // ready to show, for example "3 red flags on 2 teammates".
  label: string;
}

export interface DodgeView {
  // a last-second dodge is armed for this lobby.
  armed: boolean;
  leadMs: number;
}

// ---- data dragon champion table (command get_champions) ----

export interface ChampionEntry {
  // data dragon image id, used to build the square icon url
  id: string;
  name: string;
}

export interface ChampionData {
  version: string;
  // keyed by the stringified numeric champion id
  champions: Record<string, ChampionEntry>;
}

// ---- settings, update check and notices ----

// only op.gg and u.gg expose a shareable multi-search url; deeplol and
// tracker.gg do not, so they are not offered as scout-all providers.
export type ScoutProvider = "opgg" | "ugg";

export interface Settings {
  scoutProvider: ScoutProvider;
  autoOpen: boolean;
  revealRanked: boolean;
  launchAtLogin: boolean;
  // how long before the lobby locks in an armed last-second dodge fires.
  dodgeLeadMs: number;
  // the first-run welcome has been dismissed.
  welcomed: boolean;
}

export interface UpdateInfo {
  current: string;
  latest: string;
  available: boolean;
  url: string;
}

// a transient message pushed from the backend (an unreachable riot client) or
// raised in the frontend (a failed dodge), rendered as a toast.
export interface Notice {
  level: "info" | "error";
  message: string;
}
