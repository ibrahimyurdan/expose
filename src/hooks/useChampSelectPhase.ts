import { useEffect, useRef, useState } from "react";

import { bridge } from "@/lib/bridge";

export interface ChampSelectClock {
  phase: string;
  remainingMs: number;
}

// tracks the champ select phase and counts the remaining time down locally
// between the backend's session updates (which only arrive on changes).
export function useChampSelectPhase(): ChampSelectClock | null {
  const [clock, setClock] = useState<ChampSelectClock | null>(null);
  const endsAt = useRef(0);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    bridge
      .onPhase((phase) => {
        endsAt.current = Date.now() + phase.timeLeftMs;
        setClock({ phase: phase.phase, remainingMs: Math.max(0, phase.timeLeftMs) });
      })
      .then((fn) => {
        unlisten = fn;
      })
      .catch(() => {});

    const ticker = setInterval(() => {
      setClock((prev) =>
        prev ? { ...prev, remainingMs: Math.max(0, endsAt.current - Date.now()) } : prev,
      );
    }, 500);

    return () => {
      unlisten?.();
      clearInterval(ticker);
    };
  }, []);

  return clock;
}
