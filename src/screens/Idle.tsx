import { Users, Zap } from "lucide-react";
import { useId } from "react";

import { Card } from "@/components/ui/card";
import { Chip } from "@/components/ui/chip";
import { Toggle } from "@/components/ui/toggle";
import type { LastLobby } from "@/hooks/useLobby";
import { useMinuteClock } from "@/hooks/useMinuteClock";
import { capitalize, queueName, timeAgo } from "@/lib/format";
import type { LobbyView } from "@/types";

// connected to league, not in champ select: the auto-accept switch up front,
// what happens next, and a short recap of the last lobby once there is one.
export function Idle({
  autoAccept,
  onAutoAcceptChange,
  lastLobby,
}: {
  autoAccept: boolean;
  onAutoAcceptChange: (enabled: boolean) => void;
  lastLobby: LastLobby | null;
}) {
  const id = useId();

  return (
    // extra room below lifts the group a little over the middle, like the empty
    // states.
    <div className="flex flex-1 flex-col justify-center gap-6 px-4 pt-4 pb-16">
      <div className="flex flex-col gap-3">
        <Card className="flex items-center gap-3 p-4">
          <span
            aria-hidden
            className="flex size-9 shrink-0 items-center justify-center rounded-control bg-accent/10 text-accent"
          >
            <Zap className="size-4" />
          </span>
          <div className="min-w-0 flex-1">
            <h1 id={`${id}-title`} className="text-base font-semibold text-fg">
              Auto-accept queue pops
            </h1>
            <p id={`${id}-hint`} className="text-sm text-fg-secondary">
              {autoAccept
                ? "Expose accepts the ready check for you."
                : "Ready checks wait for you to accept them."}
            </p>
          </div>
          <Toggle
            checked={autoAccept}
            onCheckedChange={onAutoAcceptChange}
            aria-labelledby={`${id}-title`}
            aria-describedby={`${id}-hint`}
          />
        </Card>
        <p className="flex items-center gap-2 px-1 text-sm text-fg-secondary">
          <Users className="size-4 shrink-0 text-fg-muted" aria-hidden />
          When champ select starts, your teammates show up here.
        </p>
      </div>

      {lastLobby ? <LastLobbyCard last={lastLobby} /> : null}
    </div>
  );
}

function LastLobbyCard({ last }: { last: LastLobby }) {
  const id = useId();
  const now = useMinuteClock();
  const { lobby } = last;

  // everyone the lobby named besides you: matched teammates first, then names
  // the riot client listed that never got tied to a slot.
  const names = [
    ...lobby.players
      .filter((player) => !player.isSelf)
      .map((player) => player.identity?.gameName ?? ""),
    ...lobby.unmatched.map((name) => name.gameName),
  ].filter((name) => name !== "");

  return (
    <section aria-labelledby={id} className="flex flex-col gap-2">
      <div className="flex items-baseline justify-between gap-3 px-1">
        <h2 id={id} className="text-sm font-medium text-fg-secondary">
          Last lobby
        </h2>
        <span className="text-sm text-fg-secondary tabular-nums">
          {timeAgo(last.endedAt, now)}
        </span>
      </div>
      <Card className="flex flex-col gap-2 p-4">
        <div className="flex items-center justify-between gap-3">
          <span className="truncate text-base font-medium text-fg">{queueName(lobby.queueId)}</span>
          <SummaryChip lobby={lobby} />
        </div>
        <p className="truncate text-sm text-fg-secondary" title={names.join(", ")}>
          {names.length > 0 ? names.join(" · ") : "Teammate names stayed hidden."}
        </p>
      </Card>
    </section>
  );
}

// the last lobby's verdict. an all-clear in green only when every teammate's
// match history was looked through; a lobby left before that says so.
function SummaryChip({ lobby }: { lobby: LobbyView }) {
  const { summary } = lobby;
  const teammates = lobby.players.filter((player) => !player.isSelf);
  const checked = teammates.filter((player) => player.stats?.historyLoaded === true).length;
  const flagged = summary.redFlags > 0 || summary.warnings > 0;
  const all = teammates.length > 0 && checked === teammates.length;
  const tone = summary.redFlags > 0 ? "red" : summary.warnings > 0 ? "amber" : all ? "green" : "neutral";
  const label =
    flagged || all ? capitalize(summary.label) : checked === 0 ? "Not checked" : "Partly checked";
  return (
    <Chip tone={tone} title={label}>
      {label}
    </Chip>
  );
}
