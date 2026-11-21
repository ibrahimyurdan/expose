import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

// the one text button. variants follow intent: primary for the main action on a
// screen, secondary for everything else, ghost for quiet inline actions, and the
// two danger variants for leaving a lobby (soft to offer it, solid to confirm).
const buttonVariants = cva(
  "inline-flex shrink-0 items-center justify-center gap-2 whitespace-nowrap rounded-control border font-medium transition-colors disabled:pointer-events-none disabled:opacity-40 [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        primary: "border-transparent bg-accent text-on-accent hover:bg-accent-hover",
        secondary:
          "border-border-strong bg-surface-raised text-fg hover:border-fg-muted hover:bg-border",
        ghost: "border-transparent text-fg-secondary hover:bg-surface-raised hover:text-fg",
        danger: "border-red/30 bg-red-bg text-red hover:border-red/60 hover:bg-red/15",
        dangerSolid: "border-transparent bg-red text-on-accent hover:bg-red/85",
      },
      size: {
        sm: "h-7 px-2 text-sm [&_svg]:size-3.5",
        md: "h-8 px-3 text-base [&_svg]:size-4",
        lg: "h-10 px-4 text-base [&_svg]:size-4",
      },
    },
    defaultVariants: {
      variant: "secondary",
      size: "md",
    },
  },
);

type ButtonProps = ComponentProps<"button"> &
  VariantProps<typeof buttonVariants> & {
    // render the styles onto the single child instead (for example a link).
    asChild?: boolean;
  };

function Button({ className, variant, size, asChild = false, type, ...props }: ButtonProps) {
  const Comp = asChild ? Slot : "button";
  return (
    <Comp
      // a plain button by default, so one inside a form never submits it.
      type={asChild ? type : (type ?? "button")}
      className={cn(buttonVariants({ variant, size }), className)}
      {...props}
    />
  );
}

export { Button, buttonVariants };
