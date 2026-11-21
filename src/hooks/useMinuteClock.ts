import { useEffect, useState } from "react";

// the current time, refreshed every half minute: enough for "12m ago" labels.
export function useMinuteClock(): number {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const ticker = setInterval(() => setNow(Date.now()), 30_000);
    return () => clearInterval(ticker);
  }, []);
  return now;
}
