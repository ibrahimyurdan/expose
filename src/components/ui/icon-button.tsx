import type { ComponentProps } from "react";

import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";

type IconButtonProps = Omit<ComponentProps<"button">, "aria-label"> & {
  // required: there is no visible text, so this is both the accessible name
  // and the tooltip.
  label: string;
  // bare: a glyph alone (window controls, quiet inline icons). round: a gold
  // ring around a dark face (the scout button).
  variant?: "bare" | "round";
  // sm: a 24px box with a 14px glyph, for small inline icons.
  size?: "md" | "sm";
  tooltip?: boolean;
};

// a button that shows only a glyph (pass it as the child).
function IconButton({
  label,
  variant = "bare",
  size = "md",
  tooltip = true,
  className,
  type,
  children,
  ...props
}: IconButtonProps) {
  const button = (
    <button
      type={type ?? "button"}
      aria-label={label}
      // the tooltip repeats the label, so it is not announced again.
      aria-describedby={undefined}
      className={cn(
        variant === "round" ? "round-btn" : "icon-btn",
        variant === "bare" && size === "sm" && "icon-btn--sm",
        className,
      )}
      {...props}
    >
      {variant === "round" ? <span>{children}</span> : children}
    </button>
  );
  if (!tooltip) {
    return button;
  }
  return (
    <Tooltip>
      <TooltipTrigger asChild>{button}</TooltipTrigger>
      <TooltipContent>{label}</TooltipContent>
    </Tooltip>
  );
}

export { IconButton };
