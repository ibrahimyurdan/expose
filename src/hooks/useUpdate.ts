import { useEffect, useState } from "react";

import { bridge } from "@/lib/bridge";
import type { UpdateInfo } from "@/types";

// receives the result of the startup update check (current version, and whether
// a newer github release exists).
export function useUpdate(): UpdateInfo | null {
  const [info, setInfo] = useState<UpdateInfo | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    bridge
      .onUpdate(setInfo)
      .then((fn) => {
        unlisten = fn;
      })
      .catch(() => {});
    return () => unlisten?.();
  }, []);

  return info;
}
