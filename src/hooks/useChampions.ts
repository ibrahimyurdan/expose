import { useEffect, useState } from "react";

import { bridge } from "@/lib/bridge";
import type { ChampionData } from "@/types";

// loads the data dragon champion table. the backend fetches it asynchronously
// on startup, so this polls until it is available, then stops.
export function useChampions(): ChampionData | null {
  const [champions, setChampions] = useState<ChampionData | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function load(): Promise<void> {
      while (!cancelled) {
        try {
          const data = await bridge.getChampions();
          if (data) {
            if (!cancelled) {
              setChampions(data);
            }
            return;
          }
        } catch {
          // ignore and retry below
        }
        await new Promise((resolve) => setTimeout(resolve, 1500));
      }
    }

    void load();
    return () => {
      cancelled = true;
    };
  }, []);

  return champions;
}
