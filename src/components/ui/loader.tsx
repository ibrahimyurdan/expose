import { cn } from "@/lib/utils";

// three dots that bounce while something is still being worked on. the text
// beside it says what; the dots themselves are decoration.
function Loader({ className }: { className?: string }) {
  return (
    <span className={cn("loader", className)} aria-hidden>
      <i />
      <i />
      <i />
    </span>
  );
}

export { Loader };
