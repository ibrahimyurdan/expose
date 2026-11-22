import { useId } from "react";

import { Chip } from "@/components/ui/chip";
import { Frame, Panel } from "@/components/ui/frame";
import { Switch } from "@/components/ui/switch";
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
    <div className="idle">
      <div className="idle-mode">
        <Frame lips className="auto-accept">
          <div className="auto-accept-text">
            <h1 id={`${id}-title`} className="type-h5">
              Auto-accept queue pops
            </h1>
            <p id={`${id}-hint`} className="type-body">
              {autoAccept
                ? "Expose accepts the ready check for you."
                : "Ready checks wait for you to accept them."}
            </p>
          </div>
          <Switch
            checked={autoAccept}
            onCheckedChange={onAutoAcceptChange}
            aria-labelledby={`${id}-title`}
            aria-describedby={`${id}-hint`}
          />
        </Frame>
        <p className="idle-next type-body">When champ select starts, your teammates show up here.</p>
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
    <section aria-labelledby={id} className="last-lobby">
      <div className="last-lobby-head">
        <h2 id={id} className="type-subhead">
          Last lobby
        </h2>
        <span className="type-body tabular-nums">{timeAgo(last.endedAt, now)}</span>
      </div>
      <Panel className="last-lobby-panel">
        <div className="last-lobby-top">
          <span className="truncate type-h6">{queueName(lobby.queueId)}</span>
          <SummaryChip lobby={lobby} />
        </div>
        <p className="truncate type-body" title={names.join(", ")}>
          {names.length > 0 ? names.join(" · ") : "Teammate names stayed hidden."}
        </p>
      </Panel>
    </section>
  );
}

// the last lobby's verdict. an all-clear only when every teammate's match
// history was looked through; a lobby left before that says so.
function SummaryChip({ lobby }: { lobby: LobbyView }) {
  const { summary } = lobby;
  const teammates = lobby.players.filter((player) => !player.isSelf);
  const checked = teammates.filter((player) => player.stats?.historyLoaded === true).length;
  const flagged = summary.redFlags > 0 || summary.warnings > 0;
  const all = teammates.length > 0 && checked === teammates.length;
  const tone = summary.redFlags > 0 ? "red" : summary.warnings > 0 ? "amber" : "neutral";
  const label =
    flagged || all ? capitalize(summary.label) : checked === 0 ? "Not checked" : "Partly checked";
  return (
    <Chip tone={tone} warn={tone !== "neutral"} title={label}>
      {label}
    </Chip>
  );
}
