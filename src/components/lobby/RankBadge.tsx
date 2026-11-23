import { LeagueArt } from "@/components/art/LeagueArt";
import { RankCrest } from "@/components/art/RankCrest";
import { crestFile } from "@/lib/art";
import { isApexTier, tierName } from "@/lib/format";
import { cn } from "@/lib/utils";
import type { RankView } from "@/types";

// solo/duo rank: the tier's crest (the client's own, our drawing until it
// loads), the tier and division, then lp. a player without a solo/duo rank
// this season reads "Unranked" beside the unranked crest; a rank that could
// not be loaded shows nothing rather than a guess.
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
      <span className={cn("rank", className)}>
        <LeagueArt file={crestFile("unranked")} className="rank-crest" fallback={null} />
        <span className="rank-none">Unranked</span>
      </span>
    ) : null;
  }
  const tier = rank.tier.toUpperCase();
  const division = isApexTier(tier) ? "" : rank.division;
  return (
    <span className={cn("rank", className)}>
      <LeagueArt
        file={crestFile(tier)}
        className="rank-crest"
        fallback={<RankCrest tier={tier} className="rank-crest" />}
      />
      <span className="rank-tier">
        {tierName(tier)}
        {division ? ` ${division}` : null}
      </span>
      <span className="rank-lp">{rank.lp} LP</span>
    </span>
  );
}
