import { useEffect, useRef, useState, type ReactNode } from "react";

import { Header } from "@/components/shell/Header";
import { Notices } from "@/components/shell/Notices";
import { TooltipProvider } from "@/components/ui/tooltip";
import { NoticeProvider } from "@/hooks/notices";
import { useAutoAccept } from "@/hooks/useAutoAccept";
import { useChampions } from "@/hooks/useChampions";
import { useLastLobby, useLobby } from "@/hooks/useLobby";
import { useSettings } from "@/hooks/useSettings";
import { useStatus } from "@/hooks/useStatus";
import { useUpdate } from "@/hooks/useUpdate";
import { previewScreen } from "@/lib/bridge";
import { Idle } from "@/screens/Idle";
import { Lobby } from "@/screens/Lobby";
import { Settings } from "@/screens/Settings";
import { Waiting } from "@/screens/Waiting";
import { Welcome } from "@/screens/Welcome";
import type { ConnectionStatus } from "@/types";

type Screen = "loading" | "welcome" | "waiting" | "idle" | "lobby" | "settings";

export default function App() {
  return (
    <NoticeProvider>
      <TooltipProvider>
        <Shell />
      </TooltipProvider>
    </NoticeProvider>
  );
}

function Shell() {
  const status = useStatus();
  const lobby = useLobby();
  const lastLobby = useLastLobby(lobby);
  // loaded up front, so the portraits are ready the moment champ select starts.
  const champions = useChampions();
  const update = useUpdate();
  const settings = useSettings();
  const autoAccept = useAutoAccept();
  const main = useRef<HTMLElement>(null);
  // a browser preview can open straight onto settings with ?screen=settings.
  const [settingsOpen, setSettingsOpen] = useState(() => previewScreen() === "settings");

  // champ select is what matters the moment it starts, so it closes settings.
  useEffect(() => {
    if (status === "champSelect") {
      setSettingsOpen(false);
    }
  }, [status]);

  // escape leaves settings, like closing a dialog.
  useEffect(() => {
    if (!settingsOpen) {
      return;
    }
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setSettingsOpen(false);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [settingsOpen]);

  const screen = pickScreen({
    ready: status !== null && settings.loaded,
    status,
    welcomed: settings.settings.welcomed,
    settingsOpen,
  });

  // each screen starts at the top: champ select closing a scrolled settings
  // page must not open the lobby halfway down.
  useEffect(() => {
    main.current?.scrollTo({ top: 0 });
  }, [screen]);

  let content: ReactNode = null;
  switch (screen) {
    case "loading":
      break;
    case "welcome":
      content = <Welcome onStart={settings.setWelcomed} />;
      break;
    case "settings":
      content = (
        <Settings
          controls={settings}
          autoAccept={autoAccept.enabled}
          onAutoAcceptChange={autoAccept.toggle}
          update={update}
        />
      );
      break;
    case "lobby":
      content = <Lobby lobby={lobby} champions={champions} settings={settings.settings} />;
      break;
    case "idle":
      content = (
        <Idle
          autoAccept={autoAccept.enabled}
          onAutoAcceptChange={autoAccept.toggle}
          lastLobby={lastLobby}
        />
      );
      break;
    case "waiting":
      content = <Waiting />;
      break;
  }

  return (
    <div className="relative flex h-full flex-col">
      <Header
        status={status}
        update={update}
        settingsOpen={settingsOpen}
        onToggleSettings={() => setSettingsOpen((open) => !open)}
        minimal={screen === "welcome" || screen === "loading"}
      />
      {/* relative: the screen-reader-only text inside is positioned absolutely;
          main has to contain it, or it stretches the document past the window
          and scrolling a row into view slides the header away. */}
      <main ref={main} className="relative min-h-0 flex-1 overflow-x-hidden overflow-y-auto">
        <div key={screen} className="flex min-h-full animate-in flex-col duration-200 fade-in-0">
          {content}
        </div>
      </main>
      <Notices />
    </div>
  );
}

// settings, once opened, wins over everything; then champ select, which also
// postpones the first-run welcome until the lobby is over.
function pickScreen({
  ready,
  status,
  welcomed,
  settingsOpen,
}: {
  ready: boolean;
  status: ConnectionStatus | null;
  welcomed: boolean;
  settingsOpen: boolean;
}): Screen {
  if (!ready) {
    return "loading";
  }
  if (settingsOpen) {
    return "settings";
  }
  if (status === "champSelect") {
    return "lobby";
  }
  if (!welcomed) {
    return "welcome";
  }
  return status === "connected" ? "idle" : "waiting";
}
