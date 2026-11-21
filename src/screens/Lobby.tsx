import { Users } from "lucide-react";
import { useEffect, useId, useRef, useState, type RefObject } from "react";

import { LobbySummary } from "@/components/lobby/LobbySummary";
import { PlayerRow } from "@/components/lobby/PlayerRow";
import { Chip } from "@/components/ui/chip";
import { EmptyState } from "@/components/ui/empty-state";
import { useCountdown } from "@/hooks/useCountdown";
import { lobbyReceivedAt } from "@/hooks/useLobby";
import { formatClock, phaseLabel, queueName, riotId } from "@/lib/format";
import { cn } from "@/lib/utils";
import type { ChampionData, LobbyView, NameView, Settings } from "@/types";

// champ select: the clock and the verdict on your team pinned on top (with the
// dodge controls, so they stay in reach while the list scrolls), then one row
// per player in slot order, and any names the riot client listed that are not
// tied to a slot yet.
export function Lobby({
  lobby,
  champions,
  settings,
}: {
  // null for the moment between champ select starting and the first view.
  lobby: LobbyView | null;
  champions: ChampionData | null;
  settings: Settings;
}) {
  if (!lobby) {
    return (
      <EmptyState icon={Users} title="Champ select">
        Your teammates show up here in a moment.
      </EmptyState>
    );
  }
  return <ChampSelect lobby={lobby} champions={champions} settings={settings} />;
}

function ChampSelect({
  lobby,
  champions,
  settings,
}: {
  lobby: LobbyView;
  champions: ChampionData | null;
  settings: Settings;
}) {
  const scoutLabel = settings.scoutProvider === "ugg" ? "U.GG" : "OP.GG";
  const matching = lobby.anonymous && lobby.unmatched.length > 0;
  const [sentinel, stuck] = useStuck();

  // one open row at a time, forgotten when a new lobby starts.
  const [open, setOpen] = useState<{ gameId: number; key: string } | null>(null);
  const openKey = open?.gameId === lobby.gameId ? open.key : null;
  const toggle = (key: string) =>
    setOpen((current) =>
      current?.gameId === lobby.gameId && current.key === key
        ? null
        : { gameId: lobby.gameId, key },
    );

  return (
    <div className="flex flex-1 flex-col">
      <div ref={sentinel} aria-hidden className="h-px shrink-0" />
      <div
        className={cn(
          "sticky top-0 z-10 -mt-px flex flex-col gap-2 border-b bg-canvas px-4 py-2 transition-colors",
          stuck ? "border-border" : "border-transparent",
        )}
      >
        <PhaseClock lobby={lobby} />
        <LobbySummary
          lobby={lobby}
          revealRanked={settings.revealRanked}
          scoutLabel={scoutLabel}
        />
      </div>

      <div className="flex flex-col gap-4 px-4 pt-1 pb-4">
        <ul aria-label="Your team" className="flex flex-col gap-2">
          {lobby.players.map((player, index) => (
            <PlayerRow
              key={player.key}
              player={player}
              champions={champions}
              matching={matching}
              expanded={openKey === player.key}
              onToggle={() => toggle(player.key)}
              scoutLabel={scoutLabel}
              // the rows arrive one after another the first time.
              style={{ animationDelay: `${index * 40}ms` }}
            />
          ))}
        </ul>
        {lobby.unmatched.length > 0 ? <Unmatched names={lobby.unmatched} /> : null}
      </div>
    </div>
  );
}

// the phase, the queue and the time left, with a thin bar that empties as the
// current turn runs out.
function PhaseClock({ lobby }: { lobby: LobbyView }) {
  const receivedAt = lobbyReceivedAt(lobby);
  const left = useCountdown(lobby.timeLeftMs, receivedAt);
  const length = useTurnLength(lobby, receivedAt);
  const progress = length > 0 ? Math.min(1, left / length) : 0;
  // with a last-second dodge armed while locking in, a tick marks where it fires.
  const tick =
    lobby.dodge.armed && lobby.phase === "FINALIZATION" && length > 0
      ? Math.min(1, lobby.dodge.leadMs / length)
      : null;

  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex h-5 items-center justify-between gap-3">
        <p className="min-w-0 truncate text-sm">
          <span className="font-medium text-fg">{phaseLabel(lobby.phase)}</span>
          <span className="text-fg-secondary"> · {queueName(lobby.queueId)}</span>
        </p>
        <span
          role="timer"
          aria-label={`${formatClock(left)} left`}
          className="shrink-0 text-md font-semibold text-fg tabular-nums"
        >
          {formatClock(left)}
        </span>
      </div>
      <div aria-hidden className="relative h-0.5 overflow-hidden rounded-full bg-border">
        <div
          className="h-full origin-left rounded-full bg-accent transition-transform duration-300 ease-linear"
          style={{ transform: `scaleX(${progress})` }}
        />
        {tick !== null ? (
          <div className="absolute inset-y-0 w-0.5 bg-red" style={{ left: `${tick * 100}%` }} />
        ) : null}
      </div>
    </div>
  );
}

// how long the current turn of the timer is, for the progress bar. the lobby
// only reports the time left, so a turn starts with the first reading of a
// phase, or when the time left jumps back up (the next pick in pick and ban).
function useTurnLength(lobby: LobbyView, receivedAt: number): number {
  const startOf = (view: LobbyView, at: number) => ({
    view,
    gameId: view.gameId,
    phase: view.phase,
    startedAt: at,
    length: view.timeLeftMs,
  });
  const [turn, setTurn] = useState(() => startOf(lobby, receivedAt));

  if (turn.view !== lobby) {
    // what the old turn would have left by now; a reading well above it means
    // the timer restarted.
    const expected = turn.length - (receivedAt - turn.startedAt);
    const fresh =
      lobby.gameId !== turn.gameId ||
      lobby.phase !== turn.phase ||
      lobby.timeLeftMs > expected + 1000;
    const next = fresh ? startOf(lobby, receivedAt) : { ...turn, view: lobby };
    setTurn(next);
    return next.length;
  }
  return turn.length;
}

// true once the list scrolls under the pinned band, so the band can show its
// bottom edge only then.
function useStuck(): [RefObject<HTMLDivElement | null>, boolean] {
  const sentinel = useRef<HTMLDivElement | null>(null);
  const [stuck, setStuck] = useState(false);
  useEffect(() => {
    const node = sentinel.current;
    if (!node) {
      return;
    }
    const observer = new IntersectionObserver(([entry]) => setStuck(!entry?.isIntersecting));
    observer.observe(node);
    return () => observer.disconnect();
  }, []);
  return [sentinel, stuck];
}

// riot ids the riot client listed for this lobby that are not tied to a slot
// yet (ranked, while matching runs or when it could not place them).
function Unmatched({ names }: { names: NameView[] }) {
  const id = useId();
  return (
    <section aria-labelledby={id} className="flex flex-col gap-2 px-1">
      <h2 id={id} className="text-sm font-medium text-fg-secondary">
        Also in your lobby
      </h2>
      <ul className="flex flex-wrap gap-1">
        {names.map((name) => {
          const text = riotId(name);
          return (
            <li key={text} className="flex min-w-0">
              <Chip title={text}>{text}</Chip>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
