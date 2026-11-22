import type { ConnectionStatus } from "@/types";

const LABELS: Record<ConnectionStatus, string> = {
  waiting: "Waiting for League",
  connected: "Connected",
  champSelect: "Champ select",
};

const STATES: Record<ConnectionStatus, string> = {
  waiting: "waiting",
  connected: "connected",
  champSelect: "champ-select",
};

// the header's connection status: a dot and a label. grey while waiting for
// league, green once connected, teal in champ select.
function Status({ state }: { state: ConnectionStatus | null }) {
  const current = state ?? "waiting";
  return (
    <span className="status" data-state={STATES[current]} role="status">
      <span className="status-dot" aria-hidden />
      <span className="truncate">{LABELS[current]}</span>
    </span>
  );
}

export { Status };
