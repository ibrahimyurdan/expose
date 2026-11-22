import type { ComponentType, SVGProps } from "react";

import { FineGlyph, SearchGlyph, WarnGlyph } from "@/components/art/glyphs";
import { DodgeControl } from "@/components/lobby/DodgeControl";
import { Frame } from "@/components/ui/frame";
import { IconButton } from "@/components/ui/icon-button";
import { Loader } from "@/components/ui/loader";
import { usePushNotice } from "@/hooks/notices";
import { bridge } from "@/lib/bridge";
import { capitalize, plural } from "@/lib/format";
import { cn } from "@/lib/utils";
import type { LobbyView } from "@/types";

type Tone = "red" | "amber" | "neutral";

interface Verdict {
  tone: Tone;
  icon: ComponentType<SVGProps<SVGSVGElement>>;
  // the all-clear keeps a neutral card but a green check.
  clear?: boolean;
  title: string;
  detail: string;
  // the detail says teammates are still being checked: the dots bounce.
  checking?: boolean;
}

// the one-line read on the lobby, worded so it never sounds surer than the
// data: no "no red flags" while teammates are still loading or unknown.
function verdictFor(lobby: LobbyView, revealRanked: boolean): Verdict {
  const { summary } = lobby;
  const teammates = lobby.players.filter((player) => !player.isSelf);
  const loading = teammates.filter((player) => player.statsPending || player.matching).length;
  // checked means there was a match history to look through: stats that
  // failed to load raise no flags, and must not count toward an all-clear.
  const checked = teammates.filter((player) => player.stats?.historyLoaded === true).length;
  const named = teammates.filter((player) => player.identity !== null).length;

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
      icon: WarnGlyph,
      title: capitalize(summary.label),
      detail:
        stillChecking ??
        (summary.warnings > 0 ? `Plus ${plural(summary.warnings, "warning")}` : coverage),
      checking: stillChecking !== null,
    };
  }
  if (summary.warnings > 0) {
    return {
      tone: "amber",
      icon: WarnGlyph,
      title: capitalize(summary.label),
      detail: stillChecking ?? coverage,
      checking: stillChecking !== null,
    };
  }
  if (checked === 0 && teammates.length > 0) {
    if (loading > 0) {
      return {
        tone: "neutral",
        icon: SearchGlyph,
        title: "Checking your teammates",
        detail: stillChecking ?? coverage,
        checking: stillChecking !== null,
      };
    }
    return {
      tone: "neutral",
      icon: WarnGlyph,
      title: "Couldn't check your teammates",
      detail:
        !lobby.anonymous || named > 0
          ? "Their match history couldn't be loaded"
          : revealRanked
            ? "Their names couldn't be matched"
            : "Ranked matching is off in settings",
    };
  }
  return {
    tone: "neutral",
    icon: loading > 0 ? SearchGlyph : FineGlyph,
    // green only when every teammate was actually checked.
    clear: loading === 0 && checked === teammates.length,
    title: loading > 0 ? "No red flags so far" : capitalize(summary.label),
    detail: stillChecking ?? coverage,
    checking: stillChecking !== null,
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
  const Icon = verdict.icon;

  // scouting needs at least one teammate's riot id, matched or not.
  const canScout =
    lobby.unmatched.length > 0 ||
    lobby.players.some((player) => !player.isSelf && Boolean(player.identity?.gameName));

  const scout = () => {
    bridge
      .openScout()
      .catch(() => push({ level: "error", message: `Couldn't open ${scoutLabel}.` }));
  };

  return (
    <Frame
      lips
      role="region"
      aria-label="Lobby summary"
      className={cn("summary", `summary--${verdict.tone}`, verdict.clear && "summary--clear")}
    >
      <div className="summary-head">
        <Icon className="summary-icon" />
        <div className="summary-text">
          <p aria-live="polite" className="summary-title type-h5">
            {verdict.title}
          </p>
          <p className="summary-detail type-body">
            {verdict.detail}
            {verdict.checking ? <Loader /> : null}
          </p>
        </div>
        <IconButton
          label={`Scout your team on ${scoutLabel}`}
          variant="round"
          onClick={scout}
          disabled={!canScout}
        >
          <SearchGlyph />
        </IconButton>
      </div>
      <DodgeControl lobby={lobby} />
    </Frame>
  );
}
