import { cva, type VariantProps } from "class-variance-authority";
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

// a small tinted tag: red flags, amber warnings, the "likely" match marker. a
// flag's severity maps onto the tone directly, with "info" shown as neutral.
const chipVariants = cva(
  "inline-flex h-5 max-w-full shrink-0 items-center gap-1 rounded-full px-2 text-xs font-medium whitespace-nowrap [&_svg]:size-3 [&_svg]:shrink-0",
  {
    variants: {
      tone: {
        neutral: "bg-surface-raised text-fg-secondary",
        accent: "bg-accent/15 text-accent",
        red: "bg-red-bg text-red",
        amber: "bg-amber-bg text-amber",
        green: "bg-green-bg text-green",
      },
    },
    defaultVariants: {
      tone: "neutral",
    },
  },
);

type ChipTone = NonNullable<VariantProps<typeof chipVariants>["tone"]>;

type ChipProps = ComponentProps<"span"> & VariantProps<typeof chipVariants>;

// long labels truncate instead of widening the row; pass a title for the full
// text when that can happen.
function Chip({ className, tone, children, ...props }: ChipProps) {
  return (
    <span className={cn(chipVariants({ tone }), className)} {...props}>
      {typeof children === "string" ? <span className="truncate">{children}</span> : children}
    </span>
  );
}

export { Chip, chipVariants, type ChipTone };
