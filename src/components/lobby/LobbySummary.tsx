import {
  EyeOff,
  ScanSearch,
  ShieldAlert,
  ShieldCheck,
  ShieldEllipsis,
  TriangleAlert,
  type LucideIcon,
} from "lucide-react";

import { DodgeControl } from "@/components/lobby/DodgeControl";
import { IconButton } from "@/components/ui/icon-button";
import { usePushNotice } from "@/hooks/notices";
import { bridge } from "@/lib/bridge";
import { capitalize, plural } from "@/lib/format";
import { cn } from "@/lib/utils";
import type { LobbyView } from "@/types";

type Tone = "red" | "amber" | "neutral";

const TONES: Record<Tone, { card: string; icon: string }> = {
  red: { card: "border-red/25 bg-red-bg", icon: "text-red" },
  amber: { card: "border-amber/25 bg-amber-bg", icon: "text-amber" },
  neutral: { card: "border-border bg-surface", icon: "text-fg-secondary" },
};

interface Verdict {
  tone: Tone;
  icon: LucideIcon;
  // the all-clear keeps a neutral card but a green shield.
  clear?: boolean;
  title: string;
  detail: string;
}

// the one-line read on the lobby, worded so it never sounds surer than the
// data: no "no red flags" while teammates are still loading or unknown.
function verdictFor(lobby: LobbyView, revealRanked: boolean): Verdict {
  const { summary } = lobby;
  const teammates = lobby.players.filter((player) => !player.isSelf);
  const matching = lobby.anonymous && lobby.unmatched.length > 0;
  const loading = teammates.filter(
    (player) => player.statsPending || (player.identity === null && matching),
  ).length;
  const checked = teammates.filter((player) => player.stats !== null).length;

  // what the second line says when nothing more pressing needs saying. it is a
  // short status, so no full stops.
  const coverage =
    teammates.length === 0
      ? "Just you in this lobby"
      : checked < teammates.length
        ? `Checked ${checked} of ${plural(teammates.length, "teammate")}`
        : teammates.length === 1
          ? "Checked your teammate"
          : `Checked all ${teammates.length} teammates`;
  const stillChecking = loading > 0 ? `Still checking ${plural(loading, "teammate")}` : null;

  if (summary.redFlags > 0) {
    return {
      tone: "red",
      icon: ShieldAlert,
      title: capitalize(summary.label),
      detail:
        stillChecking ??
        (summary.warnings > 0 ? `Plus ${plural(summary.warnings, "warning")}` : coverage),
    };
  }
  if (summary.warnings > 0) {
    return {
      tone: "amber",
      icon: TriangleAlert,
      title: capitalize(summary.label),
      detail: stillChecking ?? coverage,
    };
  }
  if (checked === 0 && teammates.length > 0) {
    if (loading > 0) {
      return {
        tone: "neutral",
        icon: ShieldEllipsis,
        title: "Checking your teammates",
        detail: stillChecking ?? coverage,
      };
    }
    return {
      tone: "neutral",
      icon: EyeOff,
      title: "No stats for your teammates",
      detail: !lobby.anonymous
        ? "Their stats couldn't be loaded"
        : revealRanked
          ? "Their names couldn't be matched"
          : "Ranked matching is off in settings",
    };
  }
  return {
    tone: "neutral",
    icon: loading > 0 ? ShieldEllipsis : ShieldCheck,
    // green only when every teammate was actually checked.
    clear: loading === 0 && checked === teammates.length,
    title: loading > 0 ? "No red flags so far" : capitalize(summary.label),
    detail: stillChecking ?? coverage,
  };
}

// the summary bar under the clock: the verdict on your teammates, tinted by how
// bad it is, a button to scout the whole team, and the dodge controls.
export function LobbySummary({
  lobby,
  revealRanked,
  scoutLabel,
}: {
  lobby: LobbyView;
  revealRanked: boolean;
  // the scout site's name, for example "OP.GG".
  scoutLabel: string;
}) {
  const push = usePushNotice();
  const verdict = verdictFor(lobby, revealRanked);
  const tone = TONES[verdict.tone];
  const Icon = verdict.icon;

  // scouting needs at least one teammate's riot id, matched or not.
  const canScout =
    lobby.unmatched.length > 0 ||
    lobby.players.some((player) => !player.isSelf && player.identity !== null);

  const scout = () => {
    bridge
      .openScout()
      .catch(() => push({ level: "error", message: `Couldn't open ${scoutLabel}.` }));
  };

  return (
    <section
      aria-label="Lobby summary"
      className={cn("flex flex-col gap-3 rounded-card border p-3 transition-colors", tone.card)}
    >
      <div className="flex items-center gap-3">
        <Icon className={cn("size-5 shrink-0", verdict.clear ? "text-green" : tone.icon)} aria-hidden />
        <div className="min-w-0 flex-1">
          <p aria-live="polite" className="truncate text-base font-semibold text-fg">
            {verdict.title}
          </p>
          <p className="truncate text-sm text-fg-secondary">{verdict.detail}</p>
        </div>
        <IconButton
          label={`Scout your team on ${scoutLabel}`}
          variant="secondary"
          onClick={scout}
          disabled={!canScout}
        >
          <ScanSearch aria-hidden />
        </IconButton>
      </div>
      <DodgeControl lobby={lobby} />
    </section>
  );
}
