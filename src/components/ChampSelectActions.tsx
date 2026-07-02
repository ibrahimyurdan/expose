import { ExternalLink } from "lucide-react";
import { useRef, useState } from "react";

import { Button } from "@/components/ui/button";
import { usePushNotice } from "@/hooks/notices";
import { bridge } from "@/lib/bridge";
import { cn } from "@/lib/utils";
import type { Teammate } from "@/types";

// the champ-select action row: scout the whole team on the chosen site, or
// dodge. the scout url is built on the backend from the resolved team and the
// configured provider, so this just triggers it.
export function ChampSelectActions({ teammates }: { teammates: Teammate[] }) {
  const [armed, setArmed] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const push = usePushNotice();

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
    bridge
      .dodge()
      .catch(() => push({ level: "error", message: "couldn't dodge — not connected to the client" }));
  };

  const handleScout = () => {
    bridge
      .openScout()
      .catch(() => push({ level: "error", message: "couldn't open the scout page" }));
  };

  return (
    <div className="flex items-center justify-between gap-2 px-3.5 py-2.5">
      {teammates.length > 0 ? (
        <Button
          variant="outline"
          size="sm"
          onClick={handleScout}
          title="Open the whole team on your scout site"
        >
          Scout all
          <ExternalLink className="h-3 w-3" aria-hidden />
        </Button>
      ) : (
        <span className="text-[11px] text-muted-foreground">resolving teammate names…</span>
      )}
      <Button
        variant={armed ? "destructive" : "outline"}
        size="sm"
        onClick={handleDodge}
        className={cn("relative overflow-hidden", armed && "animate-pulse [animation-duration:1.1s]")}
        title={
          armed
            ? "Click again to dodge — this incurs the normal queue penalty"
            : "Dodge this lobby"
        }
      >
        {armed ? "Confirm dodge" : "Dodge"}
        {armed ? (
          <span
            aria-hidden
            className="pointer-events-none absolute bottom-0 left-0 h-0.5 w-full origin-left bg-destructive-foreground/70 [animation:dodge-drain_3s_linear_forwards]"
          />
        ) : null}
      </Button>
      <span aria-live="assertive" className="sr-only">
        {armed ? "Dodge armed. Activate again to confirm." : ""}
      </span>
    </div>
  );
}
