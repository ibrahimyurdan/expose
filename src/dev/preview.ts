// the browser stand-in for the rust backend, used when the ui runs outside
// tauri (npm run dev). it answers every bridge call from the fixtures and keeps
// toggles in memory, so each screen can be clicked through without league.
// switches, all optional:
//   ?screen=waiting|idle|lobby|settings|welcome   the screen to open (lobby)
//   ?lobby=ranked|guess|resolving|normal|live     the lobby state (ranked); live
//                                                 plays a lobby filling in: names
//                                                 first, then stats arriving
//   ?update=0|failed                              up to date, or the update
//                                                 check could not reach github
//   ?notice=1                                     raise a sample error toast

import { previewScreen, type Bridge } from "@/lib/bridge";
import {
  CHAMPIONS,
  LOBBY_VARIANTS,
  SETTINGS,
  TEAMMATE_KEYS,
  UPDATE,
  lobbyFixture,
  statusFor,
  type LobbyVariant,
} from "@/dev/fixtures";
import type { ConnectionStatus, LobbyView, Notice, Settings, UpdateInfo } from "@/types";

type Handler<T> = (payload: T) => void;

// ?lobby=live opens on the resolving lobby, then walks these steps: [delay ms,
// variant, players still loading].
const LIVE_STEPS: [number, LobbyVariant, readonly string[] | undefined][] = [
  [1500, "ranked", TEAMMATE_KEYS],
  [3000, "ranked", undefined],
  [4500, "ranked", []],
];

export function createPreviewBackend(): Bridge {
  const query = new URLSearchParams(window.location.search);
  const screen = previewScreen() ?? "lobby";
  const live = query.get("lobby") === "live";
  const variant =
    LOBBY_VARIANTS.find((candidate) => candidate === query.get("lobby")) ?? "ranked";

  let status: ConnectionStatus = statusFor(screen);
  let autoAccept = true;
  let settings: Settings = { ...SETTINGS, welcomed: screen !== "welcome" };
  // like update.rs: an empty url means the check itself failed.
  const update: UpdateInfo =
    query.get("update") === "0"
      ? { ...UPDATE, latest: UPDATE.current, available: false }
      : query.get("update") === "failed"
        ? { ...UPDATE, latest: UPDATE.current, available: false, url: "" }
        : UPDATE;

  // the lobby is rebuilt from these whenever something it shows changes.
  let shape: { variant: LobbyVariant; pending: readonly string[] | undefined } | null =
    screen === "lobby" ? { variant: live ? "resolving" : variant, pending: undefined } : null;
  let armed = false;
  let lobby: LobbyView | null = null;

  const statusHandlers = new Set<Handler<ConnectionStatus>>();
  const lobbyHandlers = new Set<Handler<LobbyView | null>>();
  const noticeHandlers = new Set<Handler<Notice>>();
  let noticeScheduled = false;

  const rebuild = () => {
    lobby = shape
      ? {
          ...lobbyFixture(shape.variant, {
            provider: settings.scoutProvider,
            pending: shape.pending,
            leadMs: settings.dodgeLeadMs,
          }),
          dodge: { armed, leadMs: settings.dodgeLeadMs },
        }
      : null;
    lobbyHandlers.forEach((handler) => handler(lobby));
  };

  const setStatus = (next: ConnectionStatus) => {
    status = next;
    statusHandlers.forEach((handler) => handler(status));
  };

  // each subscription gets its own entry, like a tauri listener id: the same
  // handler subscribed twice (strict mode remounts) must unsubscribe once.
  const subscribe = <T>(handlers: Set<Handler<T>>, handler: Handler<T>) => {
    const entry: Handler<T> = (payload) => handler(payload);
    handlers.add(entry);
    return Promise.resolve(() => {
      handlers.delete(entry);
    });
  };

  const done = () => Promise.resolve();

  // stands in for anything that would leave the app (a browser tab, a folder).
  const note = (what: string) => {
    console.info(`[preview] ${what}`);
    return done();
  };

  rebuild();
  if (live) {
    for (const [delay, step, pending] of LIVE_STEPS) {
      setTimeout(() => {
        if (shape) {
          shape = { variant: step, pending };
          rebuild();
        }
      }, delay);
    }
  }

  return {
    getStatus: () => Promise.resolve(status),
    getAutoAccept: () => Promise.resolve(autoAccept),
    setAutoAccept: (enabled) => {
      autoAccept = enabled;
      return done();
    },
    getChampions: () => Promise.resolve(CHAMPIONS),
    openExternal: (url) => note(`open ${url}`),
    // a dodge ends champ select: back to the connected state, no lobby.
    dodge: () => {
      shape = null;
      armed = false;
      rebuild();
      setStatus("connected");
      return done();
    },
    getSettings: () => Promise.resolve({ ...settings }),
    setScoutProvider: (provider) => {
      settings = { ...settings, scoutProvider: provider };
      rebuild();
      return done();
    },
    setAutoOpen: (enabled) => {
      settings = { ...settings, autoOpen: enabled };
      return done();
    },
    setRevealRanked: (enabled) => {
      settings = { ...settings, revealRanked: enabled };
      return done();
    },
    setLaunchAtLogin: (enabled) => {
      settings = { ...settings, launchAtLogin: enabled };
      return done();
    },
    openScout: () => note("open the whole team on the scout site"),
    getUpdate: () => Promise.resolve(update),
    getLobby: () => Promise.resolve(lobby),
    setLastSecondDodge: (next) => {
      armed = next && shape !== null;
      rebuild();
      return done();
    },
    setDodgeLead: (ms) => {
      settings = { ...settings, dodgeLeadMs: Math.min(3000, Math.max(1000, Math.round(ms))) };
      rebuild();
      return done();
    },
    setWelcomed: () => {
      settings = { ...settings, welcomed: true };
      return done();
    },
    openLogsFolder: () => note("open the logs folder"),

    onStatus: (handler) => subscribe(statusHandlers, handler),
    onLobby: (handler) => subscribe(lobbyHandlers, handler),
    // the update check has already finished by the time anything listens.
    onUpdate: () => Promise.resolve(() => {}),
    onNotice: (handler) => {
      // once, however often the provider resubscribes (strict mode mounts twice).
      if (query.get("notice") === "1" && !noticeScheduled) {
        noticeScheduled = true;
        setTimeout(() => {
          noticeHandlers.forEach((listener) =>
            listener({
              level: "error",
              message: "Couldn't reach the Riot client, so ranked names may stay hidden.",
            }),
          );
        }, 800);
      }
      return subscribe(noticeHandlers, handler);
    },
  };
}
