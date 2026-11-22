// a small two-tone shield for a ranked tier: the light half in the tier's
// light tint, the dark half in its dark tint. a shield rather than a gem, so
// no tier reads as diamond. tiers it does not know take gold.

const TIERS = new Set([
  "iron",
  "bronze",
  "silver",
  "gold",
  "platinum",
  "emerald",
  "diamond",
  "master",
  "grandmaster",
  "challenger",
]);

export function RankCrest({ tier, className }: { tier: string; className?: string }) {
  const name = tier.toLowerCase();
  const known = TIERS.has(name) ? name : "gold";
  return (
    <svg
      viewBox="0 0 14 14"
      className={className}
      aria-hidden
      focusable={false}
      style={{ filter: "drop-shadow(0 0 1px rgba(1, 10, 19, .8))" }}
    >
      <path d="M7 1 1.5 3V7.4Q1.5 11.2 7 13.4Z" style={{ fill: `var(--tier-${known}-hi)` }} />
      <path d="M7 1 12.5 3V7.4Q12.5 11.2 7 13.4Z" style={{ fill: `var(--tier-${known})` }} />
    </svg>
  );
}
