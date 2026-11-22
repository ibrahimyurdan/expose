import { useEffect, useId, useRef, useState, type KeyboardEvent } from "react";

import { DodgeShape } from "@/components/art/DodgeShape";
import { ClockGlyph } from "@/components/art/glyphs";
import { Switch } from "@/components/ui/switch";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { usePushNotice } from "@/hooks/notices";
import { useCountdown } from "@/hooks/useCountdown";
import { lobbyReceivedAt } from "@/hooks/useLobby";
import { armedFinal } from "@/lib/band";
import { bridge } from "@/lib/bridge";
import { formatClock, formatPreciseClock } from "@/lib/format";
import { cn } from "@/lib/utils";
import type { LobbyView } from "@/types";

// how long the armed dodge button waits for its confirming click.
const CONFIRM_MS = 3000;
// a confirming click sooner than this after arming is the second half of a
// double-click, not a decision, so it is ignored.
const CONFIRM_GUARD_MS = 600;

// the two ways out of a lobby, side by side: the "last second" switch, which
// arms a dodge the backend fires just before the lobby locks in, and the dodge
// button, which leaves now after a confirming second click.
export function DodgeControl({ lobby }: { lobby: LobbyView }) {
  return (
    <div className="dodge-row">
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
  const armedAt = useRef(0);
  const push = usePushNotice();

  useEffect(() => () => clearTimeout(timer.current), []);

  const onClick = () => {
    if (stage === "idle") {
      armedAt.current = performance.now();
      setStage("confirm");
      timer.current = setTimeout(() => setStage("idle"), CONFIRM_MS);
      return;
    }
    if (stage === "confirm" && performance.now() - armedAt.current >= CONFIRM_GUARD_MS) {
      clearTimeout(timer.current);
      setStage("leaving");
      // on success the lobby ends and this screen goes away.
      bridge
        .dodge()
        .catch(() => push({ level: "error", message: "Couldn't dodge the lobby. Try again." }))
        .finally(() => setStage("idle"));
    }
  };

  // escape backs out of the confirm step, and a held enter or space cannot
  // click through it.
  const onKeyDown = (event: KeyboardEvent<HTMLButtonElement>) => {
    if (event.repeat && (event.key === "Enter" || event.key === " ")) {
      event.preventDefault();
      return;
    }
    if (event.key === "Escape" && stage === "confirm") {
      event.stopPropagation();
      clearTimeout(timer.current);
      setStage("idle");
    }
  };

  const confirming = stage === "confirm";
  return (
    <>
      <button
        type="button"
        className="dodge"
        data-stage={stage}
        onClick={onClick}
        onKeyDown={onKeyDown}
        disabled={stage === "leaving"}
      >
        <DodgeShape />
        {stage === "idle" ? "Dodge" : stage === "confirm" ? "Confirm" : "Dodging…"}
        {confirming ? (
          <span
            aria-hidden
            className="dodge-drain"
            // drains over exactly the confirm window.
            style={{ animationDuration: `${CONFIRM_MS}ms` }}
          />
        ) : null}
      </button>
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

  const hint = `Leaves the lobby by itself with ${formatPreciseClock(lead)} left to lock in.`;

  return (
    <div className="last-second">
      <Switch
        id={`${id}-switch`}
        danger
        glyph={<ClockGlyph />}
        checked={armed}
        onCheckedChange={onChange}
        aria-label="Dodge at the last second"
        aria-describedby={`${id}-hint`}
      />
      <Tooltip>
        <TooltipTrigger asChild>
          <label
            htmlFor={`${id}-switch`}
            className={cn(
              armed ? "dodge-armed" : "type-label",
              armedFinal(lobby) && "is-final",
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
  if (!armedFinal(lobby)) {
    return <>Dodges at {formatPreciseClock(lead)}</>;
  }
  return <FinalCountdown lobby={lobby} />;
}

function FinalCountdown({ lobby }: { lobby: LobbyView }) {
  const left = useCountdown(lobby.timeLeftMs, lobbyReceivedAt(lobby));
  const until = left - lobby.dodge.leadMs;
  return <>{until > 0 ? `Dodges in ${formatClock(until)}` : "Dodging now"}</>;
}
