import { RankCrest } from "@/components/art/RankCrest";
import { isApexTier, tierName } from "@/lib/format";
import { cn } from "@/lib/utils";
import type { RankView } from "@/types";

// solo/duo rank: the tier's crest, the tier and division, then lp. a player
// without a solo/duo rank this season reads "Unranked"; a rank that could not
// be loaded shows nothing rather than a guess.
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
    return unranked ? <span className={cn("rank-none", className)}>Unranked</span> : null;
  }
  const tier = rank.tier.toUpperCase();
  const division = isApexTier(tier) ? "" : rank.division;
  return (
    <span className={cn("rank", className)}>
      <RankCrest tier={tier} />
      <span className="rank-tier">
        {tierName(tier)}
        {division ? ` ${division}` : null}
      </span>
      <span className="rank-lp">{rank.lp} LP</span>
    </span>
  );
}
