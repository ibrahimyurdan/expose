import { useCallback, useEffect, useState } from "react";

import { bridge } from "@/lib/bridge";
import type { ScoutProvider, Settings } from "@/types";

const DEFAULTS: Settings = {
  autoAccept: false,
  scoutProvider: "opgg",
  autoOpen: false,
  launchAtLogin: false,
};

// loads the persisted settings and writes changes back to the backend.
export function useSettings() {
  const [settings, setSettings] = useState<Settings>(DEFAULTS);

  useEffect(() => {
    bridge.getSettings().then(setSettings).catch(() => {});
  }, []);

  const setScoutProvider = useCallback((provider: ScoutProvider) => {
    setSettings((prev) => ({ ...prev, scoutProvider: provider }));
    bridge.setScoutProvider(provider).catch(() => {});
  }, []);

  const setAutoOpen = useCallback((enabled: boolean) => {
    setSettings((prev) => ({ ...prev, autoOpen: enabled }));
    bridge.setAutoOpen(enabled).catch(() => {});
  }, []);

  const setLaunchAtLogin = useCallback((enabled: boolean) => {
    setSettings((prev) => ({ ...prev, launchAtLogin: enabled }));
    bridge.setLaunchAtLogin(enabled).catch(() => {});
  }, []);

  return { settings, setScoutProvider, setAutoOpen, setLaunchAtLogin };
}
