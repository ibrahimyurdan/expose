// the single typed boundary to the rust backend. every invoke and event flows
// through here so components never touch raw tauri apis or stringly-typed names.
//
// outside tauri (npm run dev in a plain browser) there is no backend, so the
// same calls are answered by an in-memory stand-in built from dev/fixtures.ts.
// pick what it shows with ?screen=waiting|idle|lobby|settings|welcome (see
// dev/preview.ts for the other switches).

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  ChampionData,
  ConnectionStatus,
  LobbyView,
  Notice,
  ScoutProvider,
  Settings,
  UpdateInfo,
} from "@/types";

const EVENT_STATUS = "expose://status";
const EVENT_LOBBY = "expose://lobby";
const EVENT_UPDATE = "expose://update";
const EVENT_NOTICE = "expose://notice";
const EVENT_CHAMPIONS = "expose://champions";

export interface Bridge {
  getStatus: () => Promise<ConnectionStatus>;
  getAutoAccept: () => Promise<boolean>;
  setAutoAccept: (enabled: boolean) => Promise<void>;
  getChampions: () => Promise<ChampionData | null>;
  openExternal: (url: string) => Promise<void>;
  dodge: () => Promise<void>;
  getSettings: () => Promise<Settings>;
  setScoutProvider: (provider: ScoutProvider) => Promise<void>;
  setAutoOpen: (enabled: boolean) => Promise<void>;
  setRevealRanked: (enabled: boolean) => Promise<void>;
  setLaunchAtLogin: (enabled: boolean) => Promise<void>;
  openScout: () => Promise<void>;
  getUpdate: () => Promise<UpdateInfo | null>;
  // the latest lobby view, for a ui that mounts mid champ select.
  getLobby: () => Promise<LobbyView | null>;
  setLastSecondDodge: (armed: boolean) => Promise<void>;
  // the backend clamps this to 1000..=3000 ms.
  setDodgeLead: (ms: number) => Promise<void>;
  setWelcomed: () => Promise<void>;
  openLogsFolder: () => Promise<void>;

  onStatus: (handler: (status: ConnectionStatus) => void) => Promise<UnlistenFn>;
  // null when champ select ends.
  onLobby: (handler: (lobby: LobbyView | null) => void) => Promise<UnlistenFn>;
  onUpdate: (handler: (info: UpdateInfo) => void) => Promise<UnlistenFn>;
  onNotice: (handler: (notice: Notice) => void) => Promise<UnlistenFn>;
  // the champion table, whenever the backend loads it (startup, a new patch).
  onChampions: (handler: (champions: ChampionData) => void) => Promise<UnlistenFn>;
}

const tauriBridge: Bridge = {
  getStatus: () => invoke<ConnectionStatus>("get_status"),
  getAutoAccept: () => invoke<boolean>("get_auto_accept"),
  setAutoAccept: (enabled) => invoke<void>("set_auto_accept", { enabled }),
  getChampions: () => invoke<ChampionData | null>("get_champions"),
  openExternal: (url) => invoke<void>("open_external", { url }),
  dodge: () => invoke<void>("dodge"),
  getSettings: () => invoke<Settings>("get_settings"),
  setScoutProvider: (provider) => invoke<void>("set_scout_provider", { provider }),
  setAutoOpen: (enabled) => invoke<void>("set_auto_open", { enabled }),
  setRevealRanked: (enabled) => invoke<void>("set_reveal_ranked", { enabled }),
  setLaunchAtLogin: (enabled) => invoke<void>("set_launch_at_login", { enabled }),
  openScout: () => invoke<void>("open_scout"),
  getUpdate: () => invoke<UpdateInfo | null>("get_update"),
  getLobby: () => invoke<LobbyView | null>("get_lobby"),
  setLastSecondDodge: (armed) => invoke<void>("set_last_second_dodge", { armed }),
  // the command takes whole milliseconds (an i64).
  setDodgeLead: (ms) => invoke<void>("set_dodge_lead", { ms: Math.round(ms) }),
  setWelcomed: () => invoke<void>("set_welcomed"),
  openLogsFolder: () => invoke<void>("open_logs_folder"),

  onStatus: (handler) => listen<ConnectionStatus>(EVENT_STATUS, (event) => handler(event.payload)),
  onLobby: (handler) => listen<LobbyView | null>(EVENT_LOBBY, (event) => handler(event.payload)),
  onUpdate: (handler) => listen<UpdateInfo>(EVENT_UPDATE, (event) => handler(event.payload)),
  onNotice: (handler) => listen<Notice>(EVENT_NOTICE, (event) => handler(event.payload)),
  onChampions: (handler) =>
    listen<ChampionData>(EVENT_CHAMPIONS, (event) => handler(event.payload)),
};

// true when the ui runs in a plain browser instead of the app.
export const isPreview = !("__TAURI_INTERNALS__" in window);

export type PreviewScreen = "waiting" | "idle" | "lobby" | "settings" | "welcome";

const PREVIEW_SCREENS: readonly PreviewScreen[] = ["waiting", "idle", "lobby", "settings", "welcome"];

// the screen a browser preview was asked for with ?screen= (the lobby when the
// query names none), or null inside the app.
export function previewScreen(): PreviewScreen | null {
  if (!isPreview) {
    return null;
  }
  const asked = new URLSearchParams(window.location.search).get("screen");
  return PREVIEW_SCREENS.find((screen) => screen === asked) ?? "lobby";
}

// the browser stand-in is loaded on first use, so the app's own bundle never
// carries the fixtures.
function previewBridge(): Bridge {
  let backend: Promise<Bridge> | undefined;
  const load = (): Promise<Bridge> => {
    backend ??= import("@/dev/preview").then((module) => module.createPreviewBackend());
    return backend;
  };
  const forward = <K extends keyof Bridge>(name: K): Bridge[K] =>
    ((...args: unknown[]) =>
      load().then((stand) =>
        (stand[name] as (...rest: unknown[]) => Promise<unknown>)(...args),
      )) as Bridge[K];

  return {
    getStatus: forward("getStatus"),
    getAutoAccept: forward("getAutoAccept"),
    setAutoAccept: forward("setAutoAccept"),
    getChampions: forward("getChampions"),
    openExternal: forward("openExternal"),
    dodge: forward("dodge"),
    getSettings: forward("getSettings"),
    setScoutProvider: forward("setScoutProvider"),
    setAutoOpen: forward("setAutoOpen"),
    setRevealRanked: forward("setRevealRanked"),
    setLaunchAtLogin: forward("setLaunchAtLogin"),
    openScout: forward("openScout"),
    getUpdate: forward("getUpdate"),
    getLobby: forward("getLobby"),
    setLastSecondDodge: forward("setLastSecondDodge"),
    setDodgeLead: forward("setDodgeLead"),
    setWelcomed: forward("setWelcomed"),
    openLogsFolder: forward("openLogsFolder"),
    onStatus: forward("onStatus"),
    onLobby: forward("onLobby"),
    onUpdate: forward("onUpdate"),
    onNotice: forward("onNotice"),
    onChampions: forward("onChampions"),
  };
}

export const bridge: Bridge = isPreview ? previewBridge() : tauriBridge;
