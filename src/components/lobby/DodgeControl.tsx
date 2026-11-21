import { DoorOpen } from "lucide-react";
import { useEffect, useId, useRef, useState, type KeyboardEvent } from "react";

import { Button } from "@/components/ui/button";
import { Toggle } from "@/components/ui/toggle";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { usePushNotice } from "@/hooks/notices";
import { useCountdown } from "@/hooks/useCountdown";
import { lobbyReceivedAt } from "@/hooks/useLobby";
import { bridge } from "@/lib/bridge";
import { formatClock, formatPreciseClock, phaseLabel } from "@/lib/format";
import { cn } from "@/lib/utils";
import type { LobbyView } from "@/types";

// how long the armed dodge button waits for its confirming click.
const CONFIRM_MS = 3000;

// the two ways out of a lobby, side by side: the "last second" switch, which
// arms a dodge the backend fires just before the lobby locks in, and the dodge
// button, which leaves now after a confirming second click.
export function DodgeControl({ lobby }: { lobby: LobbyView }) {
  return (
    <div className="flex items-center gap-3">
      <LastSecond lobby={lobby} />
      <DodgeButton />
    </div>
  );
}

// leaves the lobby on the second click within three seconds, so it cannot fire
// by accident. a bar drains under the label while it waits.
function DodgeButton() {
  const [stage, setStage] = useState<"idle" | "confirm" | "leaving">("idle");
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const push = usePushNotice();

  useEffect(() => () => clearTimeout(timer.current), []);

  const onClick = () => {
    if (stage === "idle") {
      setStage("confirm");
      timer.current = setTimeout(() => setStage("idle"), CONFIRM_MS);
      return;
    }
    if (stage === "confirm") {
      clearTimeout(timer.current);
      setStage("leaving");
      // on success the lobby ends and this screen goes away.
      bridge
        .dodge()
        .catch(() => push({ level: "error", message: "Couldn't dodge the lobby. Try again." }))
        .finally(() => setStage("idle"));
    }
  };

  // escape backs out of the confirm step.
  const onKeyDown = (event: KeyboardEvent<HTMLButtonElement>) => {
    if (event.key === "Escape" && stage === "confirm") {
      event.stopPropagation();
      clearTimeout(timer.current);
      setStage("idle");
    }
  };

  const confirming = stage === "confirm";
  return (
    <>
      <Button
        variant={stage === "idle" ? "danger" : "dangerSolid"}
        onClick={onClick}
        onKeyDown={onKeyDown}
        disabled={stage === "leaving"}
        // one width for both labels, so nothing shifts while confirming.
        className="relative w-32 overflow-hidden"
      >
        {stage === "idle" ? (
          <>
            <DoorOpen aria-hidden />
            Dodge
          </>
        ) : stage === "confirm" ? (
          "Confirm dodge"
        ) : (
          "Dodging…"
        )}
        {confirming ? (
          <span
            aria-hidden
            className="pointer-events-none absolute inset-x-0 bottom-0 h-0.5 origin-left bg-on-accent/50 [animation:drain_3s_linear_forwards]"
            // drains over exactly the confirm window.
            style={{ animationDuration: `${CONFIRM_MS}ms` }}
          />
        ) : null}
      </Button>
      <span aria-live="assertive" className="sr-only">
        {confirming ? "Press again to dodge. The usual dodge penalty applies." : ""}
      </span>
    </>
  );
}

// the armed last-second dodge. the backend owns the armed state and sends it
// back in the lobby view; a click shows the new state right away and keeps it
// until the backend agrees, or reverts it if the call fails.
function LastSecond({ lobby }: { lobby: LobbyView }) {
  const id = useId();
  const push = usePushNotice();
  const [choice, setChoice] = useState<{ armed: boolean; gameId: number } | null>(null);

  // the backend caught up (or the lobby changed under the click): drop the
  // optimistic choice and show what the backend says.
  if (choice && (choice.gameId !== lobby.gameId || choice.armed === lobby.dodge.armed)) {
    setChoice(null);
  }
  const armed = choice && choice.gameId === lobby.gameId ? choice.armed : lobby.dodge.armed;
  const lead = lobby.dodge.leadMs;

  const onChange = (next: boolean) => {
    setChoice({ armed: next, gameId: lobby.gameId });
    bridge.setLastSecondDodge(next).catch(() => {
      setChoice(null);
      push({
        level: "error",
        message: next
          ? "Couldn't arm the last-second dodge."
          : "Couldn't turn off the last-second dodge. It may still fire.",
      });
    });
  };

  const hint = `Leaves the lobby by itself when ${phaseLabel("FINALIZATION")} reaches ${formatPreciseClock(lead)}.`;

  return (
    <div className="flex min-w-0 flex-1 items-center gap-2">
      <Toggle
        id={`${id}-switch`}
        checked={armed}
        onCheckedChange={onChange}
        aria-label="Dodge at the last second"
        aria-describedby={`${id}-hint`}
        className="data-[state=checked]:bg-red"
      />
      <Tooltip>
        <TooltipTrigger asChild>
          <label
            htmlFor={`${id}-switch`}
            className={cn(
              "min-w-0 truncate text-sm",
              armed ? "font-medium text-red" : "text-fg-secondary",
            )}
          >
            {armed ? <ArmedStatus lobby={lobby} /> : "Last second"}
          </label>
        </TooltipTrigger>
        <TooltipContent>{hint}</TooltipContent>
      </Tooltip>
      <span id={`${id}-hint`} className="sr-only">
        {hint}
      </span>
    </div>
  );
}

// while armed: when it fires, and once the lobby is locking in, a live
// countdown to that moment, so the dodge never comes as a surprise.
function ArmedStatus({ lobby }: { lobby: LobbyView }) {
  const lead = lobby.dodge.leadMs;
  if (lobby.phase !== "FINALIZATION") {
    return <>Dodges at {formatPreciseClock(lead)}</>;
  }
  return <FinalCountdown lobby={lobby} />;
}

function FinalCountdown({ lobby }: { lobby: LobbyView }) {
  const left = useCountdown(lobby.timeLeftMs, lobbyReceivedAt(lobby));
  const until = left - lobby.dodge.leadMs;
  return <>{until > 0 ? `Dodges in ${formatClock(until)}` : "Dodging now"}</>;
}
