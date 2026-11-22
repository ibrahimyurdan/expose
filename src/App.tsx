import { lazy, Suspense, useEffect, useRef, useState, type ReactNode } from "react";

import { TitleBar } from "@/components/shell/TitleBar";
import { WindowEdge } from "@/components/shell/WindowEdge";
import { Notices } from "@/components/shell/Notices";
import { TooltipProvider } from "@/components/ui/tooltip";
import { NoticeProvider } from "@/hooks/notices";
import { useAutoAccept } from "@/hooks/useAutoAccept";
import { useChampions } from "@/hooks/useChampions";
import { useLastLobby, useLobby } from "@/hooks/useLobby";
import { useSettings } from "@/hooks/useSettings";
import { useStatus } from "@/hooks/useStatus";
import { useUpdate } from "@/hooks/useUpdate";
import { bridge, previewScreen } from "@/lib/bridge";
import { platform } from "@/lib/platform";
import { cn } from "@/lib/utils";
import { Idle } from "@/screens/Idle";
import { Lobby } from "@/screens/Lobby";
import { Settings } from "@/screens/Settings";
import { Waiting } from "@/screens/Waiting";
import { Welcome } from "@/screens/Welcome";
import type { ConnectionStatus } from "@/types";

type Screen = "loading" | "welcome" | "waiting" | "idle" | "lobby" | "settings";

// the drawing gallery, a dev page that only the browser preview ever loads.
const Gallery = lazy(() => import("@/dev/Gallery"));

export default function App() {
  if (previewScreen() === "gallery") {
    return (
      <Suspense fallback={null}>
        <Gallery />
      </Suspense>
    );
  }
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

  const minimal = screen === "welcome" || screen === "loading";

  // ⌘, on macos and ctrl+, on windows open and close settings, as in most
  // apps. not on the welcome screen, which has no settings button either.
  useEffect(() => {
    if (minimal) {
      return;
    }
    const onKeyDown = (event: KeyboardEvent) => {
      const modifier = platform === "macos" ? event.metaKey : event.ctrlKey;
      if (modifier && event.key === ",") {
        event.preventDefault();
        setSettingsOpen((open) => !open);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [minimal]);

  // the first screen is there when the window appears, so only the screens
  // after it fade in.
  const firstShown = useRef<Screen | null>(null);
  const [fadeScreens, setFadeScreens] = useState(false);
  useEffect(() => {
    if (screen === "loading") {
      return;
    }
    if (firstShown.current === null) {
      firstShown.current = screen;
    } else if (screen !== firstShown.current) {
      setFadeScreens(true);
    }
  }, [screen]);

  // each screen starts at the top: champ select closing a scrolled settings
  // page must not open the lobby halfway down.
  useEffect(() => {
    main.current?.scrollTo({ top: 0 });
  }, [screen]);

  // once the first real screen is on the page with its fonts, tell the
  // backend, which shows the window for this launch: never empty, never with
  // the fonts swapping in. a timer rather than an animation frame, because
  // the window is still hidden and hidden pages get no frames.
  const readySent = useRef(false);
  useEffect(() => {
    if (screen === "loading" || readySent.current) {
      return;
    }
    readySent.current = true;
    void document.fonts.ready.then(() =>
      setTimeout(() => {
        bridge.ready().catch(() => {
          // the backend shows the window after a short wait anyway.
        });
      }, 0),
    );
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
      <TitleBar
        status={status}
        update={update}
        settingsOpen={settingsOpen}
        onToggleSettings={() => setSettingsOpen((open) => !open)}
        minimal={minimal}
      />
      {/* relative: the screen-reader-only text inside is positioned absolutely;
          main has to contain it, or it stretches the document past the window
          and scrolling a row into view slides the header away. */}
      <main ref={main} className="relative min-h-0 flex-1 overflow-x-hidden overflow-y-auto">
        <div
          key={screen}
          className={cn(
            "flex min-h-full flex-col",
            fadeScreens && "fade-in",
          )}
        >
          {content}
        </div>
      </main>
      <Notices />
      {platform === "windows" ? <WindowEdge /> : null}
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
