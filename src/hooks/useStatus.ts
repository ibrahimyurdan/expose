import { useEffect, useState } from "react";

import { bridge } from "@/lib/bridge";
import type { ConnectionStatus } from "@/types";

// tracks the connection status: read once on mount, then kept live by events.
export function useStatus(): ConnectionStatus {
  const [status, setStatus] = useState<ConnectionStatus>("waiting");

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    bridge.getStatus().then(setStatus).catch(() => {});
    bridge
      .onStatus(setStatus)
      .then((fn) => {
        unlisten = fn;
      })
      .catch(() => {});
    return () => unlisten?.();
  }, []);

  return status;
}
