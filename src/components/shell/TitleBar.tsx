import type { MouseEvent } from "react";

import { CloseGlyph, GearGlyph, MinimizeGlyph } from "@/components/art/glyphs";
import { Mark } from "@/components/art/Mark";
import { IconButton } from "@/components/ui/icon-button";
import { Status } from "@/components/ui/status";
import { bridge } from "@/lib/bridge";
import { platform } from "@/lib/platform";
import type { ConnectionStatus, UpdateInfo } from "@/types";

// the window's title bar, drawn by expose on both platforms: the mark and
// name, the connection status, then minimize, settings and close. the whole
// strip moves the window. the gear stays a gear while settings is open (lit,
// pressed), and a dot on it says a new version is out.
export function TitleBar({
  status,
  update,
  settingsOpen,
  onToggleSettings,
  minimal = false,
}: {
  status: ConnectionStatus | null;
  update: UpdateInfo | null;
  settingsOpen: boolean;
  onToggleSettings: () => void;
  // the welcome screen shows no settings button.
  minimal?: boolean;
}) {
  // a double click on the drag region would maximize the small panel over
  // the league client. the drag script listens on the document, so stopping
  // the double click here keeps the drag and drops the zoom (the capability
  // denies it too).
  const noZoom = (event: MouseEvent) => {
    if (event.detail >= 2) {
      event.stopPropagation();
    }
  };
  const updateNote = update?.available ? `, version ${update.latest} is available` : "";

  return (
    <header
      data-tauri-drag-region="deep"
      onMouseDown={noZoom}
      onMouseUp={noZoom}
      className="titlebar"
    >
      <div className="brand">
        <Mark size={22} />
        <span className="type-nav">Expose</span>
      </div>
      <Status state={status} />
      <div className="window-buttons">
        <IconButton
          label={platform === "macos" ? "Minimize" : "Hide (Expose keeps running in the tray)"}
          onClick={() => {
            bridge.minimizeWindow().catch(() => {});
          }}
        >
          <MinimizeGlyph />
        </IconButton>
        {!minimal && (
          <IconButton
            label={`${settingsOpen ? "Close settings" : "Settings"}${updateNote}`}
            aria-pressed={settingsOpen}
            onClick={onToggleSettings}
          >
            <span className="glyph">
              <GearGlyph />
              {update?.available ? <span className="update-dot" aria-hidden /> : null}
            </span>
          </IconButton>
        )}
        <IconButton
          label="Close"
          onClick={() => {
            bridge.hideWindow().catch(() => {});
          }}
        >
          <CloseGlyph />
        </IconButton>
      </div>
    </header>
  );
}
