import * as TooltipPrimitive from "@radix-ui/react-tooltip";
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

// a label tooltip waits a little, then fades in; once one has shown, the next
// one within half a second shows at once. a rich tooltip sets
// delayDuration={250} on its Tooltip.
function TooltipProvider({
  delayDuration = 450,
  skipDelayDuration = 500,
  ...props
}: ComponentProps<typeof TooltipPrimitive.Provider>) {
  return (
    <TooltipPrimitive.Provider
      delayDuration={delayDuration}
      skipDelayDuration={skipDelayDuration}
      {...props}
    />
  );
}

function Tooltip(props: ComponentProps<typeof TooltipPrimitive.Root>) {
  return <TooltipPrimitive.Root {...props} />;
}

const TooltipTrigger = TooltipPrimitive.Trigger;

function TooltipContent({
  className,
  sideOffset = 13,
  collisionPadding = 8,
  rich = false,
  children,
  ...props
}: ComponentProps<typeof TooltipPrimitive.Content> & {
  // a longer tooltip with a title, more padding and a bigger caret.
  rich?: boolean;
}) {
  const width = rich ? 24 : 16;
  const height = rich ? 15 : 11;
  return (
    <TooltipPrimitive.Portal>
      <TooltipPrimitive.Content
        sideOffset={sideOffset}
        collisionPadding={collisionPadding}
        className={cn("tip", rich && "tip--rich", className)}
        {...props}
      >
        {children}
        <TooltipPrimitive.Arrow asChild width={width} height={height}>
          {/* our caret: a dark v with gold sides that opens the frame where it
              points (it reaches 3px up over the edge). */}
          <svg className="tip-caret" viewBox="0 0 16 11" width={width} height={height} aria-hidden>
            <path d="M0 0H16L8 9Z" fill="var(--surface)" />
            <path d="M1 0 8 8M15 0 8 8" fill="none" stroke="#785a28" strokeWidth="2" />
            <path d="M6.4 7.2 8 9.6 9.6 7.2Z" fill="#785a28" />
          </svg>
        </TooltipPrimitive.Arrow>
      </TooltipPrimitive.Content>
    </TooltipPrimitive.Portal>
  );
}

export { Tooltip, TooltipTrigger, TooltipContent, TooltipProvider };
