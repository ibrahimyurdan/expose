import * as SwitchPrimitive from "@radix-ui/react-switch";
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

// an on/off switch. give it a name: aria-label, or aria-labelledby pointing at
// the visible label of its row.
function Toggle({ className, ...props }: ComponentProps<typeof SwitchPrimitive.Root>) {
  return (
    <SwitchPrimitive.Root
      className={cn(
        "inline-flex h-5 w-8 shrink-0 items-center rounded-full p-0.5 transition-colors disabled:opacity-40 data-[state=checked]:bg-accent data-[state=unchecked]:bg-border-strong",
        className,
      )}
      {...props}
    >
      <SwitchPrimitive.Thumb className="pointer-events-none block size-4 rounded-full bg-fg shadow-sm transition-transform data-[state=checked]:translate-x-3" />
    </SwitchPrimitive.Root>
  );
}

export { Toggle };
