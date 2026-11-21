// small display helpers shared by the screens. pure functions, no react.

import type { NameView, Position } from "@/types";

const QUEUES: Record<number, string> = {
  0: "Custom game",
  400: "Normal draft",
  420: "Ranked solo/duo",
  430: "Normal blind",
  440: "Ranked flex",
  450: "ARAM",
  490: "Quickplay",
  700: "Clash",
  720: "ARAM clash",
  830: "Co-op vs AI",
  840: "Co-op vs AI",
  850: "Co-op vs AI",
  870: "Co-op vs AI",
  880: "Co-op vs AI",
  890: "Co-op vs AI",
  900: "ARURF",
  1700: "Arena",
  1900: "URF",
};

// the player-facing name of a queue, with a neutral fallback for new ones.
export function queueName(queueId: number): string {
  return QUEUES[queueId] ?? "Champ select";
}

const PHASES: Record<string, string> = {
  PLANNING: "Declaring",
  BAN_PICK: "Pick and ban",
  FINALIZATION: "Locking in",
  GAME_STARTING: "Game starting",
};

// a champ select timer phase as the player knows it.
export function phaseLabel(phase: string): string {
  return PHASES[phase] ?? "Champ select";
}

const POSITIONS: Record<Position, string> = {
  top: "Top",
  jungle: "Jungle",
  middle: "Mid",
  bottom: "Bot",
  utility: "Support",
};

export function positionLabel(position: Position): string {
  return POSITIONS[position];
}

// a countdown as m:ss, rounding up so it reaches 0:00 only when time is out.
export function formatClock(ms: number): string {
  const total = Math.max(0, Math.ceil(ms / 1000));
  const minutes = Math.floor(total / 60);
  const seconds = total % 60;
  return `${minutes}:${String(seconds).padStart(2, "0")}`;
}

// a short duration in seconds, for example 1500 -> "1.5 s".
export function formatSeconds(ms: number): string {
  return `${Number((ms / 1000).toFixed(1))} s`;
}

// how long ago something happened, in the compact form used across the app:
// "just now", "12m ago", "2h ago", "3d ago".
export function timeAgo(then: number, now: number = Date.now()): string {
  const minutes = Math.floor((now - then) / 60_000);
  if (minutes < 1) {
    return "just now";
  }
  if (minutes < 60) {
    return `${minutes}m ago`;
  }
  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return `${hours}h ago`;
  }
  return `${Math.floor(hours / 24)}d ago`;
}

// the first letter in upper case, for labels the backend words in lower case
// ("no red flags") that open a sentence on screen.
export function capitalize(text: string): string {
  return text.charAt(0).toUpperCase() + text.slice(1);
}

// a count with its noun: "1 teammate", "3 teammates".
export function plural(count: number, word: string): string {
  return `${count} ${word}${count === 1 ? "" : "s"}`;
}

// a riot id the way players type it, "name#tag" (just the name without a tag).
export function riotId(name: NameView): string {
  return name.tagLine ? `${name.gameName}#${name.tagLine}` : name.gameName;
}

// a red flag's label as its chip shows it: sentence case, and kda in capitals
// the way players write it (the backend words labels in lower case).
export function flagText(label: string): string {
  return capitalize(label.replace(/\bkda\b/gi, "KDA"));
}

const TIERS: Record<string, string> = {
  IRON: "Iron",
  BRONZE: "Bronze",
  SILVER: "Silver",
  GOLD: "Gold",
  PLATINUM: "Platinum",
  EMERALD: "Emerald",
  DIAMOND: "Diamond",
  MASTER: "Master",
  GRANDMASTER: "Grandmaster",
  CHALLENGER: "Challenger",
};

// a ranked tier as the client names it: "GOLD" -> "Gold".
export function tierName(tier: string): string {
  return TIERS[tier.toUpperCase()] ?? capitalize(tier.toLowerCase());
}

// master and above have no divisions, only lp.
export function isApexTier(tier: string): boolean {
  return ["MASTER", "GRANDMASTER", "CHALLENGER"].includes(tier.toUpperCase());
}

// a clock reading that keeps tenths when there are any: 1500 -> "0:01.5",
// 2000 -> "0:02". used for the last-second dodge, where half a second counts.
export function formatPreciseClock(ms: number): string {
  const tenths = Math.max(0, Math.round(ms / 100));
  const total = Math.floor(tenths / 10);
  const minutes = Math.floor(total / 60);
  const seconds = total % 60;
  const fraction = tenths % 10;
  return `${minutes}:${String(seconds).padStart(2, "0")}${fraction ? `.${fraction}` : ""}`;
}

// a game length in whole minutes: 1860 -> "31m".
export function formatMinutes(seconds: number): string {
  return `${Math.max(1, Math.round(seconds / 60))}m`;
}

// a share from 0 to 1 as a whole percentage: 0.534 -> "53%".
export function formatPercent(share: number): string {
  return `${Math.round(share * 100)}%`;
}

// a kda ratio with one decimal: 3.14 -> "3.1".
export function formatKda(kda: number): string {
  return kda.toFixed(1);
}

// champion mastery points, short: 9500 -> "9.5k", 186400 -> "186k",
// 1250000 -> "1.3M".
export function formatPoints(points: number): string {
  // from here on the thousands would round up to "1000k".
  if (points >= 999_500) {
    return `${Number((points / 1_000_000).toFixed(1))}M`;
  }
  if (points >= 100_000) {
    return `${Math.round(points / 1000)}k`;
  }
  if (points >= 1000) {
    return `${Number((points / 1000).toFixed(1))}k`;
  }
  return String(points);
}
