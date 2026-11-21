import { isApexTier, tierName } from "@/lib/format";
import { cn } from "@/lib/utils";
import type { RankView } from "@/types";

// written out in full so tailwind finds every tier color class in the source.
const TIER_COLORS: Record<string, string> = {
  IRON: "text-tier-iron",
  BRONZE: "text-tier-bronze",
  SILVER: "text-tier-silver",
  GOLD: "text-tier-gold",
  PLATINUM: "text-tier-platinum",
  EMERALD: "text-tier-emerald",
  DIAMOND: "text-tier-diamond",
  MASTER: "text-tier-master",
  GRANDMASTER: "text-tier-grandmaster",
  CHALLENGER: "text-tier-challenger",
};

// solo/duo rank: a small diamond and the tier in the tier's color, then lp.
// a player without a solo/duo rank this season reads "Unranked"; a rank that
// could not be loaded shows nothing rather than a guess.
export function RankBadge({
  rank,
  unranked,
  className,
}: {
  rank: RankView | null;
  unranked: boolean;
  className?: string;
}) {
  if (!rank) {
    return unranked ? (
      <span className={cn("text-sm whitespace-nowrap text-fg-secondary", className)}>Unranked</span>
    ) : null;
  }
  const tier = rank.tier.toUpperCase();
  const color = TIER_COLORS[tier] ?? "text-fg";
  const division = isApexTier(tier) ? "" : rank.division;
  return (
    <span className={cn("inline-flex items-center gap-1.5 text-sm whitespace-nowrap", className)}>
      <span aria-hidden className={cn("size-1.5 shrink-0 rotate-45 rounded-[1px] bg-current", color)} />
      <span className={cn("font-medium", color)}>
        {tierName(tier)}
        {division ? ` ${division}` : null}
      </span>
      <span className="text-fg-secondary tabular-nums">{rank.lp} LP</span>
    </span>
  );
}
