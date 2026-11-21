import { useEffect, useState } from "react";

import { bridge } from "@/lib/bridge";
import type { ConnectionStatus } from "@/types";

// tracks the connection status: read once on mount, then kept live by events.
// null until the first answer, so the shell can wait instead of flashing the
// wrong screen.
export function useStatus(): ConnectionStatus | null {
  const [status, setStatus] = useState<ConnectionStatus | null>(null);

  useEffect(() => {
    let cancelled = false;
    let heard = false;
    let unlisten: (() => void) | undefined;
    // subscribe before reading, and never let the read overwrite a newer event.
    bridge
      .onStatus((next) => {
        heard = true;
        setStatus(next);
      })
      .then((fn) => {
        if (cancelled) {
          fn();
          return;
        }
        unlisten = fn;
        return bridge.getStatus().then((value) => {
          if (!cancelled && !heard) {
            setStatus(value);
          }
        });
      })
      // with no backend to ask, treat league as not found rather than wait forever.
      .catch(() => {
        if (!cancelled && !heard) {
          setStatus("waiting");
        }
      });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  return status;
}
