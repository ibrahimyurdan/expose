import type { ComponentProps } from "react";

import { WarnGlyph } from "@/components/art/glyphs";
import { cn } from "@/lib/utils";

export type ChipTone = "red" | "amber" | "neutral" | "info" | "soft";

// the notice pill for flags: red and amber flags carry the warning glyph.
// long labels truncate instead of widening the row; pass a title for the full
// text when that can happen. other props reach the span, so a chip can be a
// tooltip's trigger.
function Chip({
  tone,
  warn = false,
  className,
  children,
  ...props
}: ComponentProps<"span"> & { tone: ChipTone; warn?: boolean }) {
  return (
    <span className={cn("chip", `chip--${tone}`, className)} {...props}>
      {warn && <WarnGlyph />}
      <span>{children}</span>
    </span>
  );
}

export type TagTone = "gold" | "amber" | "teal" | "red";

// a small square tag, for "likely".
function Tag({ tone, className, ...props }: ComponentProps<"span"> & { tone: TagTone }) {
  return <span className={cn("tag", `tag--${tone}`, className)} {...props} />;
}

export { Chip, Tag };
