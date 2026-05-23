import { Settings as SettingsIcon, X } from "lucide-react";

import type { ChampSelectClock } from "@/hooks/useChampSelectPhase";
import { cn } from "@/lib/utils";
import type { ConnectionStatus } from "@/types";

const STATUS_META: Record<ConnectionStatus, { label: string; dot: string }> = {
  waiting: { label: "Waiting for League", dot: "bg-status-waiting" },
  connected: { label: "Connected", dot: "bg-status-connected" },
  champSelect: { label: "In champ select", dot: "bg-status-active" },
};

const PHASE_LABELS: Record<string, string> = {
  PLANNING: "Declaring",
  BAN_PICK: "Pick / ban",
  FINALIZATION: "Locking in",
};

function formatClock(ms: number): string {
  const total = Math.round(ms / 1000);
  const minutes = Math.floor(total / 60);
  const seconds = total % 60;
  return `${minutes}:${String(seconds).padStart(2, "0")}`;
}

export function StatusHeader({
  status,
  clock,
  settingsOpen,
  onToggleSettings,
}: {
  status: ConnectionStatus;
  clock: ChampSelectClock | null;
  settingsOpen: boolean;
  onToggleSettings: () => void;
}) {
  const meta = STATUS_META[status];
  const label =
    status === "champSelect" && clock
      ? `${PHASE_LABELS[clock.phase] ?? "Champ select"} · ${formatClock(clock.remainingMs)}`
      : meta.label;

  return (
    <header className="flex items-center justify-between px-4 py-3">
      <div className="flex items-center gap-2">
        <span className="text-base leading-none text-gold" aria-hidden>
          ◆
        </span>
        <span className="hex-label text-sm text-gold-bright">Expose</span>
      </div>
      <div className="flex items-center gap-2">
        {settingsOpen ? (
          <span className="hex-label text-[11px] text-muted-foreground">Settings</span>
        ) : (
          <>
            <span
              className={cn(
                "h-2 w-2 rounded-full",
                meta.dot,
                status === "waiting" && "animate-pulse",
              )}
              aria-hidden
            />
            <span className="hex-label text-[11px] text-muted-foreground">{label}</span>
          </>
        )}
        <button
          type="button"
          onClick={onToggleSettings}
          className="text-muted-foreground transition-colors hover:text-gold"
          aria-label={settingsOpen ? "Close settings" : "Open settings"}
          title="Settings"
        >
          {settingsOpen ? <X className="h-4 w-4" /> : <SettingsIcon className="h-4 w-4" />}
        </button>
      </div>
    </header>
  );
}
