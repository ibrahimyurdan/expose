import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

// a calm, centered message for a screen with nothing to show yet: an icon in a
// soft circle, a title, one line of explanation and optional actions below.
function EmptyState({
  icon: Icon,
  title,
  children,
  actions,
  className,
}: {
  icon: LucideIcon;
  title: string;
  children?: ReactNode;
  actions?: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn(
        // more room below than above sits the group a little over the middle,
        // where it reads as centered.
        "flex flex-1 flex-col items-center justify-center gap-4 px-8 pt-10 pb-20 text-center",
        className,
      )}
    >
      <span className="flex size-14 items-center justify-center rounded-full border border-border bg-surface text-fg-secondary">
        <Icon className="size-6" strokeWidth={1.75} aria-hidden />
      </span>
      <div className="flex max-w-72 flex-col gap-1 text-balance">
        <h1 className="text-md font-semibold text-fg">{title}</h1>
        {children ? <p className="text-base text-fg-secondary">{children}</p> : null}
      </div>
      {actions ? <div className="flex items-center gap-2">{actions}</div> : null}
    </div>
  );
}

export { EmptyState };
