import { useEffect, useState } from "react";

import { bridge } from "@/lib/bridge";
import type { Teammate } from "@/types";

// the resolved teammate list, pushed from the backend on every champ select
// update and cleared (empty array) when a session ends.
export function useTeammates(): Teammate[] {
  const [teammates, setTeammates] = useState<Teammate[]>([]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    bridge
      .onTeammates(setTeammates)
      .then((fn) => {
        unlisten = fn;
      })
      .catch(() => {});
    return () => unlisten?.();
  }, []);

  return teammates;
}
