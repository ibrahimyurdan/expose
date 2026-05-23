import { ExternalLink } from "lucide-react";
import { useRef, useState } from "react";

import { Button } from "@/components/ui/button";
import { bridge } from "@/lib/bridge";
import type { Teammate } from "@/types";

// the champ-select action row: scout the whole team on the chosen site, or
// dodge. the scout url is built on the backend from the resolved team and the
// configured provider, so this just triggers it.
export function ChampSelectActions({ teammates }: { teammates: Teammate[] }) {
  const [armed, setArmed] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  // a deliberate two-step dodge so it can't fire by accident.
  const handleDodge = () => {
    if (!armed) {
      setArmed(true);
      timer.current = setTimeout(() => setArmed(false), 3000);
      return;
    }
    if (timer.current) {
      clearTimeout(timer.current);
    }
    setArmed(false);
    bridge.dodge().catch(() => {});
  };

  return (
    <div className="flex items-center justify-between gap-2 px-4 py-2">
      {teammates.length > 0 ? (
        <Button
          variant="outline"
          size="sm"
          onClick={() => bridge.openScout().catch(() => {})}
          title="Open the whole team on your scout site"
        >
          Scout all
          <ExternalLink className="h-3 w-3" />
        </Button>
      ) : (
        <span className="text-[11px] text-muted-foreground">resolving teammates…</span>
      )}
      <Button variant={armed ? "destructive" : "outline"} size="sm" onClick={handleDodge}>
        {armed ? "Confirm dodge" : "Dodge"}
      </Button>
    </div>
  );
}
