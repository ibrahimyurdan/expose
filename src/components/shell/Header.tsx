import { Eye, Settings as SettingsIcon, X } from "lucide-react";
import type { MouseEvent } from "react";

import { IconButton } from "@/components/ui/icon-button";
import { Status } from "@/components/ui/status";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { bridge } from "@/lib/bridge";
import { platform } from "@/lib/platform";
import type { ConnectionStatus, UpdateInfo } from "@/types";

// the 44px bar across the top: the wordmark, the connection state, an update
// notice when a newer release exists, and the settings button. on macos the
// traffic lights sit over its left edge, so it starts 72px in there and doubles
// as the window's drag handle (buttons inside still click).
export function Header({
  status,
  update,
  settingsOpen,
  onToggleSettings,
  minimal = false,
}: {
  // null until the first answer from the backend; the pill waits for it.
  status: ConnectionStatus | null;
  update: UpdateInfo | null;
  settingsOpen: boolean;
  onToggleSettings: () => void;
  // the welcome screen shows only the wordmark and the connection state.
  minimal?: boolean;
}) {
  const mac = platform === "macos";
  // tauri zooms the window when its drag region is double-clicked, which
  // would blow this small always-on-top panel up over the league client. the
  // drag script listens on the document, so stopping the double-click here
  // keeps the drag and drops the zoom.
  const noZoom = (event: MouseEvent) => {
    if (event.detail >= 2) {
      event.stopPropagation();
    }
  };

  return (
    <header
      data-tauri-drag-region={mac ? "deep" : undefined}
      onMouseDown={mac ? noZoom : undefined}
      onMouseUp={mac ? noZoom : undefined}
      className="flex h-11 shrink-0 items-center gap-3 border-b border-border px-4 mac:pl-18"
    >
      <div className="flex shrink-0 items-center gap-2">
        <Eye className="size-4 text-accent" strokeWidth={2.25} aria-hidden />
        <span className="text-base font-semibold tracking-tight text-fg">Expose</span>
      </div>

      <div className="ml-auto flex min-w-0 items-center gap-2">
        <Status state={status} />

        {!minimal && update?.available ? (
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                type="button"
                className="btn-text"
                onClick={() => bridge.openExternal(update.url).catch(() => {})}
                aria-label={`Update to version ${update.latest}`}
              >
                Update
              </button>
            </TooltipTrigger>
            <TooltipContent>Version {update.latest} is available</TooltipContent>
          </Tooltip>
        ) : null}

        {!minimal ? (
          <IconButton
            label={settingsOpen ? "Close settings" : "Settings"}
            aria-expanded={settingsOpen}
            onClick={onToggleSettings}
            className="-mr-2"
          >
            {settingsOpen ? <X aria-hidden /> : <SettingsIcon aria-hidden />}
          </IconButton>
        ) : null}
      </div>
    </header>
  );
}
