import { useCallback, useEffect, useState } from "react";

import { bridge } from "@/lib/bridge";

// the auto-accept toggle. the backend owns the persisted value; this mirrors it
// optimistically and writes changes back through the command.
export function useAutoAccept(): { enabled: boolean; toggle: (next: boolean) => void } {
  const [enabled, setEnabled] = useState(false);

  useEffect(() => {
    bridge.getAutoAccept().then(setEnabled).catch(() => {});
  }, []);

  const toggle = useCallback((next: boolean) => {
    setEnabled(next);
    bridge.setAutoAccept(next).catch(() => {});
  }, []);

  return { enabled, toggle };
}
