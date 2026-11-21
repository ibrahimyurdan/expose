import { useEffect, useRef, useState } from "react";

import { bridge } from "@/lib/bridge";
import type { LobbyView } from "@/types";

// when each lobby view arrived, kept beside the view (which mirrors the backend
// exactly) so its phase timer can count down locally between events.
const arrivals = new WeakMap<LobbyView, number>();

// when a lobby view arrived, as a Date.now() timestamp, for useCountdown. a
// view that only repeats the previous view's timer (the backend re-sends the
// lobby as stats come in) keeps the earlier arrival, so a countdown never jumps
// back up.
export function lobbyReceivedAt(view: LobbyView): number {
  let at = arrivals.get(view);
  if (at === undefined) {
    at = Date.now();
    arrivals.set(view, at);
  }
  return at;
}

function sameTimer(a: LobbyView, b: LobbyView): boolean {
  return a.gameId === b.gameId && a.phase === b.phase && a.timeLeftMs === b.timeLeftMs;
}

// the champ select lobby, or null outside champ select. read once on mount
// (the window can open mid lobby), then kept live by the lobby event.
export function useLobby(): LobbyView | null {
  const [lobby, setLobby] = useState<LobbyView | null>(null);
  const previous = useRef<LobbyView | null>(null);

  useEffect(() => {
    let cancelled = false;
    let heard = false;
    let unlisten: (() => void) | undefined;

    const accept = (view: LobbyView | null) => {
      const before = previous.current;
      if (view) {
        arrivals.set(
          view,
          before && sameTimer(before, view) ? lobbyReceivedAt(before) : Date.now(),
        );
      }
      previous.current = view;
      setLobby(view);
    };

    // subscribe before reading so an update in between cannot be lost, and
    // never let the initial read overwrite a newer pushed view.
    bridge
      .onLobby((view) => {
        heard = true;
        accept(view);
      })
      .then((fn) => {
        if (cancelled) {
          fn();
          return;
        }
        unlisten = fn;
        return bridge.getLobby().then((view) => {
          if (!cancelled && !heard) {
            accept(view);
          }
        });
      })
      .catch(() => {});

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  return lobby;
}

export interface LastLobby {
  lobby: LobbyView;
  endedAt: number;
}

// the most recent lobby that has ended (dodged, or the game started), kept for
// the idle screen's summary. null until a lobby has come and gone.
export function useLastLobby(lobby: LobbyView | null): LastLobby | null {
  const [last, setLast] = useState<LastLobby | null>(null);
  const latest = useRef<LobbyView | null>(null);

  useEffect(() => {
    if (lobby) {
      latest.current = lobby;
      return;
    }
    if (latest.current) {
      setLast({ lobby: latest.current, endedAt: Date.now() });
      latest.current = null;
    }
  }, [lobby]);

  return last;
}
