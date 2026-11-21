import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

// a bordered surface that groups related content: a settings section, the
// auto-accept control, a teammate row.
function Card({ className, ...props }: ComponentProps<"div">) {
  return <div className={cn("rounded-card border border-border bg-surface", className)} {...props} />;
}

export { Card };
