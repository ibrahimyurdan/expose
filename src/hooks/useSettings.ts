import { useCallback, useEffect, useState } from "react";

import { bridge } from "@/lib/bridge";
import type { ScoutProvider, Settings } from "@/types";

// used until the backend answers, and for any field an older backend leaves
// out. welcomed defaults to true so a failed read never traps anyone on the
// welcome screen.
const DEFAULTS: Settings = {
  scoutProvider: "opgg",
  autoOpen: false,
  revealRanked: true,
  launchAtLogin: false,
  dodgeLeadMs: 1500,
  welcomed: true,
};

// loads the persisted settings and writes changes back to the backend. each
// change shows immediately; `loaded` turns true once the first read settles.
export function useSettings() {
  const [settings, setSettings] = useState<Settings>(DEFAULTS);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    let cancelled = false;
    bridge
      .getSettings()
      .then((value) => {
        if (!cancelled) {
          setSettings({ ...DEFAULTS, ...value });
        }
      })
      .catch(() => {})
      .finally(() => {
        if (!cancelled) {
          setLoaded(true);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const setScoutProvider = useCallback((provider: ScoutProvider) => {
    setSettings((prev) => ({ ...prev, scoutProvider: provider }));
    bridge.setScoutProvider(provider).catch(() => {});
  }, []);

  const setAutoOpen = useCallback((enabled: boolean) => {
    setSettings((prev) => ({ ...prev, autoOpen: enabled }));
    bridge.setAutoOpen(enabled).catch(() => {});
  }, []);

  const setRevealRanked = useCallback((enabled: boolean) => {
    setSettings((prev) => ({ ...prev, revealRanked: enabled }));
    bridge.setRevealRanked(enabled).catch(() => {});
  }, []);

  // the os can refuse this one, so the toggle flips back and the promise
  // rejects for the caller to explain.
  const setLaunchAtLogin = useCallback((enabled: boolean): Promise<void> => {
    setSettings((prev) => ({ ...prev, launchAtLogin: enabled }));
    return bridge.setLaunchAtLogin(enabled).catch((error: unknown) => {
      setSettings((prev) => ({ ...prev, launchAtLogin: !enabled }));
      throw error;
    });
  }, []);

  const setDodgeLead = useCallback((ms: number) => {
    setSettings((prev) => ({ ...prev, dodgeLeadMs: ms }));
    bridge.setDodgeLead(ms).catch(() => {});
  }, []);

  const setWelcomed = useCallback(() => {
    setSettings((prev) => ({ ...prev, welcomed: true }));
    bridge.setWelcomed().catch(() => {});
  }, []);

  return {
    settings,
    loaded,
    setScoutProvider,
    setAutoOpen,
    setRevealRanked,
    setLaunchAtLogin,
    setDodgeLead,
    setWelcomed,
  };
}
