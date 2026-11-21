import { cva, type VariantProps } from "class-variance-authority";
import type { ComponentProps } from "react";

import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";

const iconButtonVariants = cva(
  "inline-flex shrink-0 items-center justify-center rounded-control border transition-colors disabled:pointer-events-none disabled:opacity-40 [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        ghost: "border-transparent text-fg-secondary hover:bg-surface-raised hover:text-fg",
        secondary:
          "border-border-strong bg-surface-raised text-fg-secondary hover:border-fg-muted hover:text-fg",
      },
      size: {
        xs: "size-6 [&_svg]:size-3.5",
        sm: "size-7 [&_svg]:size-4",
        md: "size-8 [&_svg]:size-4",
      },
    },
    defaultVariants: {
      variant: "ghost",
      size: "md",
    },
  },
);

type IconButtonProps = Omit<ComponentProps<"button">, "aria-label"> &
  VariantProps<typeof iconButtonVariants> & {
    // required: there is no visible text, so this is both the accessible name
    // and the hover and focus hint.
    label: string;
  };

// a square button that shows only an icon (pass the lucide icon as the child).
function IconButton({ label, variant, size, className, type, ...props }: IconButtonProps) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <button
          type={type ?? "button"}
          aria-label={label}
          // the tooltip repeats the label, so it should not also be announced
          // as a description.
          aria-describedby={undefined}
          className={cn(iconButtonVariants({ variant, size }), className)}
          {...props}
        />
      </TooltipTrigger>
      <TooltipContent>{label}</TooltipContent>
    </Tooltip>
  );
}

export { IconButton, iconButtonVariants };
