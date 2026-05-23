import { ScrollArea } from "@/components/ui/scroll-area";
import { TeammateRow } from "@/components/TeammateRow";
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
      <div className="flex flex-col gap-2 p-3">
        {teammates.map((teammate) => (
          <TeammateRow
            key={teammate.puuid || teammate.cellId}
            teammate={teammate}
            champions={champions}
          />
        ))}
      </div>
    </ScrollArea>
  );
}

function EmptyState({ status }: { status: ConnectionStatus }) {
  const { title, hint } = messageFor(status);
  return (
    <div className="flex h-full flex-col items-center justify-center gap-2 px-8 text-center">
      <span className="text-2xl text-gold-deep" aria-hidden>
        ◆
      </span>
      <p className="hex-label text-sm text-foreground">{title}</p>
      <p className="text-xs text-muted-foreground">{hint}</p>
    </div>
  );
}

function messageFor(status: ConnectionStatus): { title: string; hint: string } {
  switch (status) {
    case "waiting":
      return {
        title: "Waiting for League",
        hint: "Start the League client and Expose will connect automatically.",
      };
    case "connected":
      return {
        title: "Connected",
        hint: "Teammates appear here once ranked champ select begins.",
      };
    case "champSelect":
      return { title: "Champ select", hint: "Resolving teammate names…" };
  }
}
