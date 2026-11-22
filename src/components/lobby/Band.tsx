import type { CSSProperties, ReactNode } from "react";

import { MetalWork } from "@/components/art/MetalWork";
import { useCountdown } from "@/hooks/useCountdown";
import { lobbyReceivedAt } from "@/hooks/useLobby";
import { armedFinal, barModes, formatBandClock, type BarMode } from "@/lib/band";
import { phaseLabel, queueName } from "@/lib/format";
import { cn } from "@/lib/utils";
import type { LobbyView } from "@/types";

// the band pinned at the top of champ select: the phase, the clock between
// two bars that drain toward it, framed in gold metal-work, and the queue.
// the bars follow the stage: teal for planning, lock-in and your team's
// picks, red for bans and the enemy's picks, a side hidden while it waits.
// the children (the verdict card) stay pinned with it.
export function Band({
  lobby,
  stuck,
  children,
}: {
  lobby: LobbyView;
  // the team has scrolled under the band: a fade hangs under it.
  stuck: boolean;
  children?: ReactNode;
}) {
  const left = useCountdown(lobby.timeLeftMs, lobbyReceivedAt(lobby));
  const total = lobby.totalMs > 0 ? lobby.totalMs : Math.max(lobby.timeLeftMs, 1);
  const final = armedFinal(lobby);
  const modes = barModes(lobby.stage, lobby.acting, final);
  const yourTurn = lobby.players.some((player) => player.isSelf && player.acting);
  const fill = Math.min(1, Math.max(0, left / total));
  // an armed dodge fires this far from the clock end of each bar.
  const tick = final ? Math.min(1, lobby.dodge.leadMs / total) : null;

  return (
    <div
      className={cn(
        "band",
        stuck && "is-stuck",
        left > 0 && left <= 10_000 && "is-final",
      )}
    >
      <div className="band-head">
        <p className={cn("band-phase", yourTurn ? "type-phase-turn" : "type-phase")}>
          {phaseLabel(lobby.phase)}
        </p>
        <div className="band-clock-row">
          <MetalWork side="left" className="metal metal--left" />
          <Bar side="left" mode={modes.left} fill={fill} tick={tick} />
          <span
            role="timer"
            aria-label={`${Math.ceil(left / 1000)} seconds left`}
            className="band-clock type-h3"
          >
            {formatBandClock(left, lobby.totalMs)}
          </span>
          <Bar side="right" mode={modes.right} fill={fill} tick={tick} />
          <MetalWork side="right" className="metal metal--right" />
        </div>
        <p className="band-queue type-queue">{queueName(lobby.queueId)}</p>
      </div>
      {children}
    </div>
  );
}

function Bar({
  side,
  mode,
  fill,
  tick,
}: {
  side: "left" | "right";
  mode: BarMode;
  fill: number;
  tick: number | null;
}) {
  const style = { "--p": fill } as CSSProperties;
  return (
    <span className={cn("burnout", `burnout--${side}`, `is-${mode}`)} aria-hidden>
      <i style={style} />
      <i style={style} />
      {tick !== null ? (
        <span
          className="tick"
          style={side === "left" ? { right: `${tick * 100}%` } : { left: `${tick * 100}%` }}
        />
      ) : null}
    </span>
  );
}
