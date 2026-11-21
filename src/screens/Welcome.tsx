import { CircleCheck, DoorOpen, Eye, Users, type LucideIcon } from "lucide-react";

import { Button } from "@/components/ui/button";

const POINTS: { icon: LucideIcon; title: string; text: string }[] = [
  {
    icon: Users,
    title: "See who you're queued with",
    text: "Rank, recent games and red flags for every teammate, even when ranked hides their names.",
  },
  {
    icon: CircleCheck,
    title: "Auto-accept ready checks",
    text: "Step away while you queue. Expose accepts the match for you.",
  },
  {
    icon: DoorOpen,
    title: "Dodge now or at the last second",
    text: "Leave a lobby with two clicks, or arm a dodge that fires just before it locks in.",
  },
];

// the first-run screen: what expose does, in three lines, and one way forward.
// it shows until "get started" is pressed once.
export function Welcome({ onStart }: { onStart: () => void }) {
  return (
    <div className="mx-auto flex w-full max-w-sm flex-1 flex-col justify-center gap-8 px-6 pt-8 pb-16">
      <div className="flex flex-col items-center gap-4 text-center">
        <span
          aria-hidden
          className="flex size-14 items-center justify-center rounded-full bg-accent/10 text-accent"
        >
          <Eye className="size-7" strokeWidth={2} />
        </span>
        <div className="flex flex-col gap-1">
          <h1 className="text-lg font-semibold text-fg">Welcome to Expose</h1>
          <p className="text-base text-fg-secondary">
            Your champ select companion for League of Legends.
          </p>
        </div>
      </div>

      <ul className="flex flex-col gap-4">
        {POINTS.map(({ icon: Icon, title, text }) => (
          <li key={title} className="flex gap-3">
            <span
              aria-hidden
              className="flex size-8 shrink-0 items-center justify-center rounded-control border border-border bg-surface text-fg-secondary"
            >
              <Icon className="size-4" />
            </span>
            <div className="min-w-0">
              <p className="text-base font-medium text-fg">{title}</p>
              <p className="text-sm text-fg-secondary">{text}</p>
            </div>
          </li>
        ))}
      </ul>

      <Button variant="primary" size="lg" className="w-full" onClick={onStart}>
        Get started
      </Button>
    </div>
  );
}
