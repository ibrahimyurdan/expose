// the champ select band's logic: how its clock reads and which of its two
// bars show in which color. only type imports, so node runs the tests in
// tests/ui straight from source.

import type { Acting, Stage } from "@/types";

// teal: a turn on your side, or planning and lock-in. red: bans, the enemy's
// turn, or an armed dodge's final seconds. hidden: that side is not acting.
export type BarMode = "teal" | "red" | "hidden";

// an armed dodge is in its final seconds (red bars, the tick, the live
// countdown) only while the lobby locks in: FINALIZATION, the one phase the
// backend fires in. GAME_STARTING shares the finalization stage, but it is
// past the dodge, so it must not say "dodging now".
export function armedFinal(lobby: { phase: string; dodge: { armed: boolean } }): boolean {
  return lobby.dodge.armed && lobby.phase === "FINALIZATION";
}

// bare seconds for a phase under 100 s, as the client prints them; minutes
// and seconds for a longer one. an unknown length goes by the time left.
export function formatBandClock(leftMs: number, totalMs: number): string {
  const total = totalMs > 0 ? totalMs : leftMs;
  const seconds = Math.max(0, Math.ceil(leftMs / 1000));
  if (total < 100_000) {
    return String(seconds);
  }
  const minutes = Math.floor(seconds / 60);
  return `${minutes}:${String(seconds % 60).padStart(2, "0")}`;
}

// the left bar is your team's, the right one the enemy's.
export function barModes(
  stage: Stage,
  acting: Acting,
  armedFinal: boolean,
): { left: BarMode; right: BarMode } {
  if (armedFinal) {
    return { left: "red", right: "red" };
  }
  if ((stage !== "ban" && stage !== "pick") || acting === "none") {
    return { left: "teal", right: "teal" };
  }
  const ally = acting === "ally" || acting === "both";
  const enemy = acting === "enemy" || acting === "both";
  return {
    left: ally ? (stage === "ban" ? "red" : "teal") : "hidden",
    right: enemy ? "red" : "hidden",
  };
}
