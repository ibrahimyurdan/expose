import { useEffect, useState } from "react";

import { ScrollArea } from "@/components/ui/scroll-area";
import { TeammateRow } from "@/components/TeammateRow";
import { cn } from "@/lib/utils";
import type { ChampionData, ConnectionStatus, Teammate } from "@/types";

export function TeammateList({
  status,
  teammates,
  champions,
}: {
  status: ConnectionStatus;
  teammates: Teammate[];
  champions: ChampionData | null;
}) {
  if (teammates.length === 0) {
    return <EmptyState status={status} />;
  }

  return (
    <ScrollArea className="h-full">
      <div className="flex flex-col gap-2 px-3.5 py-2.5">
        {teammates.map((teammate, index) => (
          <div
            key={teammate.puuid || teammate.cellId}
            className="animate-in fade-in-0 slide-in-from-left-2 fill-mode-backwards duration-300"
            style={{ animationDelay: `${index * 45}ms` }}
          >
            <TeammateRow teammate={teammate} champions={champions} />
          </div>
        ))}
      </div>
    </ScrollArea>
  );
}

function EmptyState({ status }: { status: ConnectionStatus }) {
  const stalled = useResolutionStalled(status);
  const { title, hint, glyph } = messageFor(status, stalled);
  return (
    <div className="flex h-full animate-in flex-col items-center justify-center gap-2 px-8 text-center duration-300 fade-in-0 slide-in-from-bottom-1">
      <span className={cn("leading-none", glyph)} aria-hidden>
        ◆
      </span>
      <p className="hex-label text-sm text-foreground">{title}</p>
      <p className="text-xs text-muted-foreground">{hint}</p>
    </div>
  );
}

// in champ select an empty team usually means names are still resolving — but if
// it stays empty it has actually failed (no riot client, reveal off). flip to a
// terminal message after a short grace period so it stops reading as a hang.
function useResolutionStalled(status: ConnectionStatus): boolean {
  const [stalled, setStalled] = useState(false);
  useEffect(() => {
    setStalled(false);
    if (status !== "champSelect") {
      return;
    }
    const timer = setTimeout(() => setStalled(true), 7000);
    return () => clearTimeout(timer);
  }, [status]);
  return stalled;
}

function messageFor(
  status: ConnectionStatus,
  stalled: boolean,
): { title: string; hint: string; glyph: string } {
  switch (status) {
    case "waiting":
      return {
        title: "Waiting for League",
        hint: "Start the League client and Expose will connect automatically.",
        glyph: "text-4xl text-gold/40 animate-pulse [animation-duration:2.5s]",
      };
    case "connected":
      return {
        title: "Connected",
        hint: "Teammates appear here once champ select begins.",
        glyph: "text-3xl text-status-connected",
      };
    case "champSelect":
      return stalled
        ? {
            title: "No teammate names",
            hint: "Make sure the Riot client is running, or check reveal in settings.",
            glyph: "text-3xl text-gold-deep",
          }
        : {
            title: "Champ select",
            hint: "Resolving teammate names…",
            glyph: "text-3xl text-gold animate-spin [animation-duration:1.4s]",
          };
  }
}
