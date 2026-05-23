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
  opggUrl: string;
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

export type ScoutProvider = "opgg" | "ugg" | "deeplol" | "tracker";

export interface Settings {
  autoAccept: boolean;
  scoutProvider: ScoutProvider;
  autoOpen: boolean;
  launchAtLogin: boolean;
}

export interface UpdateInfo {
  current: string;
  latest: string;
  available: boolean;
  url: string;
}
