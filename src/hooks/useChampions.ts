import { useEffect, useState } from "react";

import { bridge } from "@/lib/bridge";
import type { ChampionData } from "@/types";

// the data dragon champion table. the backend loads it in the background, so
// this asks once for what is there and then listens for each load (startup,
// or a new patch when league connects).
export function useChampions(): ChampionData | null {
  const [champions, setChampions] = useState<ChampionData | null>(null);

  useEffect(() => {
    let cancelled = false;
    const unlisten = bridge.onChampions((data) => {
      if (!cancelled) {
        setChampions(data);
      }
    });
    // asked only once the listener is in place, so a load that lands in
    // between is never missed.
    unlisten
      .then(() => bridge.getChampions())
      .then((data) => {
        // a load that arrived through the event first wins.
        if (!cancelled && data) {
          setChampions((current) => current ?? data);
        }
      })
      .catch(() => {
        // the event brings the table once the backend has it.
      });
    return () => {
      cancelled = true;
      void unlisten.then((stop) => stop());
    };
  }, []);

  return champions;
}
