import { cn } from "@/lib/utils";
import type { GameView } from "@/types";

// recent results as a row of dots, newest on the left: teal for a win, dark
// red for a loss (the bright red stays for flags). screen readers get the
// count instead of ten separate dots.
export function FormDots({
  games,
  limit = 10,
  className,
}: {
  // newest first, as the backend sends them.
  games: GameView[];
  limit?: number;
  className?: string;
}) {
  const shown = games.slice(0, limit);
  const wins = shown.filter((game) => game.win).length;
  return (
    <span className={cn("inline-flex shrink-0 items-center", className)}>
      <span className="sr-only">
        Last {shown.length} games: {wins} won, {shown.length - wins} lost.
      </span>
      <span aria-hidden className="dots">
        {shown.map((game, index) => (
          <i key={`${game.startedAtMs}-${index}`} className={game.win ? "win" : "loss"} />
        ))}
      </span>
    </span>
  );
}
