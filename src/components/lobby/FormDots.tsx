import { cn } from "@/lib/utils";
import type { GameView } from "@/types";

// recent results as a row of dots, newest on the left: green for a win, red
// for a loss. screen readers get the count instead of ten separate dots.
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
      <span aria-hidden className="inline-flex items-center gap-0.5">
        {shown.map((game, index) => (
          <span
            key={`${game.startedAtMs}-${index}`}
            className={cn("size-1.5 rounded-full", game.win ? "bg-green" : "bg-red")}
          />
        ))}
      </span>
    </span>
  );
}
