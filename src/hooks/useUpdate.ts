import { useEffect, useState } from "react";

import { bridge } from "@/lib/bridge";
import type { UpdateInfo } from "@/types";

// the result of the startup update check (current version, and whether a newer
// github release exists): read once on mount, since the check usually finishes
// before the settings panel opens, then kept live by the event.
export function useUpdate(): UpdateInfo | null {
  const [info, setInfo] = useState<UpdateInfo | null>(null);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    // subscribe before reading: the backend stores the result before it emits,
    // so this order cannot miss a check that finishes in between.
    bridge
      .onUpdate(setInfo)
      .then((fn) => {
        if (cancelled) {
          fn();
          return;
        }
        unlisten = fn;
        return bridge.getUpdate().then((value) => {
          if (!cancelled && value) setInfo(value);
        });
      })
      .catch(() => {});
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  return info;
}
