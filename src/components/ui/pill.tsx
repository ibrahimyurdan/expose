import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

// a rounded status indicator, like the connection state in the header: a short
// label with an optional colored dot. it renders a span; pass asChild to put
// the same look on a button.
const pillVariants = cva(
  "inline-flex h-6 min-w-0 shrink-0 items-center gap-1.5 rounded-full border px-2 text-xs font-medium whitespace-nowrap transition-colors [&_svg]:size-3.5 [&_svg]:shrink-0",
  {
    variants: {
      tone: {
        neutral: "border-border bg-surface text-fg-secondary",
        accent: "border-accent/30 bg-accent/10 text-accent",
        red: "border-red/30 bg-red-bg text-red",
        amber: "border-amber/30 bg-amber-bg text-amber",
        green: "border-green/30 bg-green-bg text-green",
      },
    },
    defaultVariants: {
      tone: "neutral",
    },
  },
);

const DOT_COLORS = {
  muted: "bg-fg-muted",
  accent: "bg-accent",
  red: "bg-red",
  amber: "bg-amber",
  green: "bg-green",
} as const;

type PillProps = ComponentProps<"span"> &
  VariantProps<typeof pillVariants> & {
    asChild?: boolean;
    // a leading status dot in its own color, independent of the pill's tone.
    dot?: keyof typeof DOT_COLORS;
    // a slow pulse on the dot, for "still looking" states.
    pulse?: boolean;
  };

function Pill({ className, tone, asChild = false, dot, pulse, children, ...props }: PillProps) {
  // a slot takes exactly one child, so the dot is only drawn on the plain span.
  if (asChild) {
    return (
      <Slot className={cn(pillVariants({ tone }), className)} {...props}>
        {children}
      </Slot>
    );
  }
  return (
    <span className={cn(pillVariants({ tone }), className)} {...props}>
      {dot ? (
        <span
          aria-hidden
          className={cn(
            "size-1.5 shrink-0 rounded-full",
            DOT_COLORS[dot],
            pulse && "animate-pulse [animation-duration:2.4s]",
          )}
        />
      ) : null}
      <span className="truncate">{children}</span>
    </span>
  );
}

export { Pill, pillVariants };
