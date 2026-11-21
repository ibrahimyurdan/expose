import { CircleAlert, Info, X } from "lucide-react";

import { IconButton } from "@/components/ui/icon-button";
import { useToasts } from "@/hooks/notices";
import { cn } from "@/lib/utils";

// the toast stack, floating over the bottom of the window. the live region
// stays mounted while empty, so a screen reader hears each new message.
export function Notices() {
  const { toasts, dismiss } = useToasts();
  return (
    <div
      aria-live="polite"
      aria-relevant="additions"
      className="pointer-events-none absolute inset-x-0 bottom-0 z-40 flex flex-col items-center gap-2 p-4"
    >
      {toasts.map((toast) => {
        const error = toast.level === "error";
        const Icon = error ? CircleAlert : Info;
        return (
          <div
            key={toast.id}
            className="pointer-events-auto flex w-full max-w-sm animate-in items-start gap-2 rounded-control border border-border-strong bg-surface-raised py-2 pr-2 pl-3 shadow-popover duration-200 fade-in-0 slide-in-from-bottom-2"
          >
            {/* the icon and the first line of text center on the 24px dismiss button */}
            <Icon
              className={cn("mt-1 size-4 shrink-0", error ? "text-red" : "text-accent")}
              aria-hidden
            />
            <p className="min-w-0 flex-1 py-1 text-sm text-fg">{toast.message}</p>
            <IconButton label="Dismiss" size="xs" onClick={() => dismiss(toast.id)}>
              <X aria-hidden />
            </IconButton>
          </div>
        );
      })}
    </div>
  );
}
