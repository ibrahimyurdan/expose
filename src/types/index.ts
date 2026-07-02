// these mirror the serde payloads emitted by the rust backend. keep them in
// sync with src-tauri/src/models.rs.

export type ConnectionStatus = "waiting" | "connected" | "champSelect";

export interface Teammate {
  cellId: number;
  puuid: string;
  gameName: string;
  tagLine: string;
  summonerLevel: number;
  profileIconId: number;
  assignedPosition: string;
  championId: number;
  championPickIntent: number;
  // single-summoner profile link on the chosen scout site, with that site's
  // label for the row button.
  scoutUrl: string;
  scoutLabel: string;
}

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

export interface ChampSelectPhase {
  // raw lcu phase, e.g. PLANNING, BAN_PICK, FINALIZATION
  phase: string;
  timeLeftMs: number;
}

// only op.gg and u.gg expose a shareable multi-search url; deeplol and
// tracker.gg do not, so they are not offered as scout-all providers.
export type ScoutProvider = "opgg" | "ugg";

export interface Settings {
  scoutProvider: ScoutProvider;
  autoOpen: boolean;
  revealRanked: boolean;
  launchAtLogin: boolean;
}

export interface UpdateInfo {
  current: string;
  latest: string;
  available: boolean;
  url: string;
}

// a transient message pushed from the backend (a failed resolution) or raised in
// the frontend (a failed dodge), rendered as a toast.
export interface Notice {
  level: "info" | "error";
  message: string;
}
