import { CloseGlyph, WarnGlyph } from "@/components/art/glyphs";
import { useToasts } from "@/hooks/notices";
import { cn } from "@/lib/utils";

// the toasts, framed cards at the bottom of the window that fade in and out.
// the live region stays mounted while empty, so a screen reader hears each
// new message.
export function Notices() {
  const { toasts, dismiss } = useToasts();
  return (
    <div
      aria-live="polite"
      aria-relevant="additions"
      className="pointer-events-none absolute inset-x-0 bottom-0 z-40 flex flex-col items-center gap-5 pb-5"
    >
      {toasts.map((toast) => {
        const error = toast.level === "error";
        return (
          <div
            key={toast.id}
            data-leaving={toast.leaving || undefined}
            className={cn("toast pointer-events-auto", error && "toast--error")}
          >
            <span className="frame-lips" aria-hidden />
            <div className="toast-body">
              {error && <WarnGlyph />}
              <p className="type-body text-gold-1">{toast.message}</p>
            </div>
            <button
              type="button"
              className="toast-close"
              aria-label="Dismiss"
              onClick={() => dismiss(toast.id)}
            >
              <CloseGlyph />
            </button>
          </div>
        );
      })}
    </div>
  );
}
