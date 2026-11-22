import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";

import { bridge } from "@/lib/bridge";
import type { Notice } from "@/types";

export interface Toast extends Notice {
  id: number;
  // fading out, about to go.
  leaving?: boolean;
}

// errors stay a little longer than plain information, so they can be read.
const LIFETIME_MS = { info: 4000, error: 7000 } as const;
// how long a toast takes to fade out (see .toast[data-leaving]).
const FADE_MS = 400;

const PushContext = createContext<(notice: Notice) => void>(() => {});
const ToastContext = createContext<{ toasts: Toast[]; dismiss: (id: number) => void }>({
  toasts: [],
  dismiss: () => {},
});

// lets any component raise a transient toast (for example a failed dodge).
export function usePushNotice(): (notice: Notice) => void {
  return useContext(PushContext);
}

// the toasts on screen, for components/shell/Notices.tsx to render.
export function useToasts(): { toasts: Toast[]; dismiss: (id: number) => void } {
  return useContext(ToastContext);
}

// owns the active toasts and forwards backend notices into the same list, so
// backend events and frontend failures share one consistent surface. pushing
// and reading are separate contexts, so components that only push do not
// re-render whenever a toast comes or goes.
export function NoticeProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const nextId = useRef(0);
  const timers = useRef(new Map<number, ReturnType<typeof setTimeout>>());

  // fades the toast out, then removes it.
  const dismiss = useCallback((id: number) => {
    clearTimeout(timers.current.get(id));
    setToasts((current) =>
      current.map((toast) => (toast.id === id ? { ...toast, leaving: true } : toast)),
    );
    timers.current.set(
      id,
      setTimeout(() => {
        timers.current.delete(id);
        setToasts((current) => current.filter((toast) => toast.id !== id));
      }, FADE_MS),
    );
  }, []);

  const push = useCallback(
    (notice: Notice) => {
      const id = nextId.current++;
      setToasts((current) => {
        // the same message twice in a row is one toast, not a stack.
        if (current.some((toast) => toast.message === notice.message && !toast.leaving)) {
          return current;
        }
        // at most two at once, so a burst cannot cover the window.
        return [...current.slice(-1), { ...notice, id }];
      });
      timers.current.set(
        id,
        setTimeout(() => dismiss(id), LIFETIME_MS[notice.level]),
      );
    },
    [dismiss],
  );

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    bridge
      .onNotice(push)
      .then((fn) => {
        if (cancelled) {
          fn();
        } else {
          unlisten = fn;
        }
      })
      .catch(() => {});
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [push]);

  useEffect(() => {
    const pending = timers.current;
    return () => pending.forEach((timer) => clearTimeout(timer));
  }, []);

  const toastValue = useMemo(() => ({ toasts, dismiss }), [toasts, dismiss]);

  return (
    <PushContext.Provider value={push}>
      <ToastContext.Provider value={toastValue}>{children}</ToastContext.Provider>
    </PushContext.Provider>
  );
}
