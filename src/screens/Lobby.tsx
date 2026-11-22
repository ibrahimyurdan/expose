import { useEffect, useId, useRef, useState, type RefObject } from "react";

import { Band } from "@/components/lobby/Band";
import { LobbySummary } from "@/components/lobby/LobbySummary";
import { PlayerRow } from "@/components/lobby/PlayerRow";
import { Chip } from "@/components/ui/chip";
import { EmptyState } from "@/components/ui/empty-state";
import { riotId } from "@/lib/format";
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
      <EmptyState title="Champ select">
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
      <Band lobby={lobby} stuck={stuck}>
        <LobbySummary
          lobby={lobby}
          revealRanked={settings.revealRanked}
          scoutLabel={scoutLabel}
        />
      </Band>

      <div className="flex flex-col gap-4 px-4 pt-1 pb-4">
        <ul aria-label="Your team" className="flex flex-col gap-2">
          {lobby.players.map((player, index) => (
            <PlayerRow
              key={player.key}
              player={player}
              champions={champions}
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
              <Chip tone="neutral" title={text}>
                {text}
              </Chip>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
