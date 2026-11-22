import { useEffect, useRef, useState, type ComponentProps, type MouseEvent } from "react";

import { ExternalGlyph } from "@/components/art/glyphs";
import { cn } from "@/lib/utils";

// framed: the default button, a gold edge on a dark face. primary: the one
// main action on a screen, on a bronze face. thin: small actions (open the
// logs, view a profile). text: a quiet display label with no frame.
export type ButtonVariant = "framed" | "primary" | "thin" | "text";

const CLASSES: Record<ButtonVariant, string> = {
  framed: "btn",
  primary: "btn btn--primary",
  thin: "btn-thin",
  text: "btn-text",
};

// after a click the button holds its pressed look and ignores input for a
// moment, so a double click never fires twice.
const CLICK_LOCK_MS: Record<ButtonVariant, number> = {
  framed: 600,
  primary: 600,
  thin: 250,
  text: 0,
};

type ButtonProps = ComponentProps<"button"> & {
  variant?: ButtonVariant;
  // the label leaves the app: an arrow follows it.
  external?: boolean;
};

function Button({
  variant = "framed",
  external = false,
  className,
  type,
  onClick,
  children,
  ...props
}: ButtonProps) {
  const [clicked, setClicked] = useState(false);
  const timer = useRef<number | undefined>(undefined);
  useEffect(() => () => window.clearTimeout(timer.current), []);

  const lock = CLICK_LOCK_MS[variant];
  const handleClick = (event: MouseEvent<HTMLButtonElement>) => {
    // css blocks the pointer while the press shows; this blocks the keyboard.
    if (clicked) {
      event.preventDefault();
      return;
    }
    onClick?.(event);
    if (lock > 0) {
      setClicked(true);
      window.clearTimeout(timer.current);
      timer.current = window.setTimeout(() => setClicked(false), lock);
    }
  };
  const framed = variant === "framed" || variant === "primary";

  return (
    <button
      // a plain button by default, so one inside a form never submits it.
      type={type ?? "button"}
      className={cn(CLASSES[variant], clicked && "is-clicked", className)}
      onClick={handleClick}
      {...props}
    >
      {framed && <span className="btn-fill" aria-hidden />}
      {framed && <span className="btn-glow" aria-hidden />}
      {children}
      {external && <ExternalGlyph className="ext" />}
    </button>
  );
}

export { Button };
