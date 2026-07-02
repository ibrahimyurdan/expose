// the single typed boundary to the rust backend. every invoke and event flows
// through here so components never touch raw tauri apis or stringly-typed names.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  ChampionData,
  ChampSelectPhase,
  ConnectionStatus,
  Notice,
  ScoutProvider,
  Settings,
  Teammate,
  UpdateInfo,
} from "@/types";

const EVENT_STATUS = "expose://status";
const EVENT_TEAMMATES = "expose://teammates";
const EVENT_PHASE = "expose://phase";
const EVENT_UPDATE = "expose://update";
const EVENT_NOTICE = "expose://notice";

export const bridge = {
  getStatus: () => invoke<ConnectionStatus>("get_status"),
  getAutoAccept: () => invoke<boolean>("get_auto_accept"),
  setAutoAccept: (enabled: boolean) => invoke<void>("set_auto_accept", { enabled }),
  getChampions: () => invoke<ChampionData | null>("get_champions"),
  openExternal: (url: string) => invoke<void>("open_external", { url }),
  dodge: () => invoke<void>("dodge"),
  getSettings: () => invoke<Settings>("get_settings"),
  setScoutProvider: (provider: ScoutProvider) =>
    invoke<void>("set_scout_provider", { provider }),
  setAutoOpen: (enabled: boolean) => invoke<void>("set_auto_open", { enabled }),
  setRevealRanked: (enabled: boolean) => invoke<void>("set_reveal_ranked", { enabled }),
  setLaunchAtLogin: (enabled: boolean) => invoke<void>("set_launch_at_login", { enabled }),
  openScout: () => invoke<void>("open_scout"),

  onStatus: (handler: (status: ConnectionStatus) => void): Promise<UnlistenFn> =>
    listen<ConnectionStatus>(EVENT_STATUS, (event) => handler(event.payload)),
  onTeammates: (handler: (teammates: Teammate[]) => void): Promise<UnlistenFn> =>
    listen<Teammate[]>(EVENT_TEAMMATES, (event) => handler(event.payload)),
  onPhase: (handler: (phase: ChampSelectPhase) => void): Promise<UnlistenFn> =>
    listen<ChampSelectPhase>(EVENT_PHASE, (event) => handler(event.payload)),
  onUpdate: (handler: (info: UpdateInfo) => void): Promise<UnlistenFn> =>
    listen<UpdateInfo>(EVENT_UPDATE, (event) => handler(event.payload)),
  onNotice: (handler: (notice: Notice) => void): Promise<UnlistenFn> =>
    listen<Notice>(EVENT_NOTICE, (event) => handler(event.payload)),
};
