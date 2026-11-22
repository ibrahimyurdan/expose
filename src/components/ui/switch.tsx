import * as SwitchPrimitive from "@radix-ui/react-switch";
import type { ComponentProps, ReactNode } from "react";

import { cn } from "@/lib/utils";

type SwitchProps = ComponentProps<typeof SwitchPrimitive.Root> & {
  // red when on, for the last-second dodge; green otherwise.
  danger?: boolean;
  // a small glyph on the knob.
  glyph?: ReactNode;
};

// the switch for a live mode (auto-accept, the last-second dodge); settings
// use checkboxes. give it a name: aria-label, or aria-labelledby pointing at
// the visible label beside it.
function Switch({ className, danger = false, glyph, ...props }: SwitchProps) {
  return (
    <SwitchPrimitive.Root className={cn("switch", danger && "switch--danger", className)} {...props}>
      <span className="switch-track" aria-hidden>
        <svg className="switch-check" viewBox="0 0 12 9" focusable={false}>
          <path d="M1 4.5 4.3 7.8 11 1.2" fill="none" stroke="currentColor" strokeWidth="2" />
        </svg>
      </span>
      <SwitchPrimitive.Thumb className="switch-knob">{glyph}</SwitchPrimitive.Thumb>
    </SwitchPrimitive.Root>
  );
}

export { Switch };
