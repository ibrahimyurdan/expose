import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

type FrameProps = ComponentProps<"div"> & {
  // the edge that shines: bottom for a card with actions under it, top for
  // one that points at something above it.
  bright?: "bottom" | "top";
  // the trim lines just outside the top and bottom edges.
  lips?: boolean;
};

// a card in a gold frame.
function Frame({ bright = "bottom", lips = false, className, children, ...props }: FrameProps) {
  return (
    <div className={cn("frame", bright === "top" && "frame--top", className)} {...props}>
      {lips && <span className="frame-lips" aria-hidden />}
      {children}
    </div>
  );
}

// a quiet container with a thin dark edge.
function Panel({ className, ...props }: ComponentProps<"div">) {
  return <div className={cn("panel", className)} {...props} />;
}

// a raised card on a lighter face, bright at the top: the details panel.
function Raised({ className, ...props }: ComponentProps<"div">) {
  return <div className={cn("raised", className)} {...props} />;
}

export { Frame, Panel, Raised };
