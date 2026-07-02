import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";

import { bridge } from "@/lib/bridge";
import { cn } from "@/lib/utils";
import type { Notice } from "@/types";

interface Toast extends Notice {
  id: number;
}

const PushContext = createContext<(notice: Notice) => void>(() => {});

// lets any component raise a transient toast (for example a failed dodge).
export function usePushNotice(): (notice: Notice) => void {
  return useContext(PushContext);
}

// owns the active toasts, forwards backend notices into the same surface, and
// renders the stack. one provider wraps the app so both backend events and
// frontend-raised failures flow through a single, consistent toast.
export function NoticeProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const nextId = useRef(0);

  const push = useCallback((notice: Notice) => {
    const id = nextId.current++;
    // keep at most three on screen so a burst cannot fill the tiny window.
    setToasts((current) => [...current.slice(-2), { ...notice, id }]);
    setTimeout(() => {
      setToasts((current) => current.filter((toast) => toast.id !== id));
    }, 4000);
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    bridge
      .onNotice(push)
      .then((fn) => {
        unlisten = fn;
      })
      .catch(() => {});
    return () => unlisten?.();
  }, [push]);

  return (
    <PushContext.Provider value={push}>
      {children}
      <Toaster toasts={toasts} />
    </PushContext.Provider>
  );
}

function Toaster({ toasts }: { toasts: Toast[] }) {
  if (toasts.length === 0) {
    return null;
  }
  return (
    <div className="pointer-events-none absolute inset-x-0 bottom-[68px] z-50 flex flex-col items-center gap-1.5 px-4">
      {toasts.map((toast) => (
        <div
          key={toast.id}
          role="status"
          className="hex-frame flex max-w-full animate-in items-center gap-2 rounded-sm px-3 py-1.5 text-xs shadow-lg duration-200 fade-in-0 slide-in-from-bottom-2"
        >
          <span
            aria-hidden
            className={cn(
              "shrink-0 text-[10px] leading-none",
              toast.level === "error" ? "text-destructive" : "text-gold",
            )}
          >
            ◆
          </span>
          <span className="text-foreground">{toast.message}</span>
        </div>
      ))}
    </div>
  );
}
