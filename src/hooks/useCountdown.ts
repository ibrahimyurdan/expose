import { useEffect, useState } from "react";

// counts a phase timer down locally between lobby events, which only arrive
// when something changes. `ms` is the time left when the view arrived at
// `receivedAt` (a Date.now() timestamp; see lobbyReceivedAt). the result ticks
// every 250 ms, stops at zero, and restarts whenever a new reading comes in.
// it re-renders its caller on every tick, so call it from a small component.
export function useCountdown(ms: number, receivedAt: number): number {
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    const left = (at: number) => ms - (at - receivedAt);
    const start = Date.now();
    setNow(start);
    if (left(start) <= 0) {
      return;
    }
    const ticker = setInterval(() => {
      const current = Date.now();
      setNow(current);
      if (left(current) <= 0) {
        clearInterval(ticker);
      }
    }, 250);
    return () => clearInterval(ticker);
  }, [ms, receivedAt]);

  // a reading newer than the last tick can make now - receivedAt negative, so
  // cap at ms as well as at zero.
  return Math.max(0, Math.min(ms, ms - (now - receivedAt)));
}
