import { EmptyState } from "@/components/ui/empty-state";
import { Loader } from "@/components/ui/loader";

// league is not running (or expose has not found it yet). nothing to do here:
// the supervisor keeps looking and the title bar's status changes once it
// connects.
export function Waiting() {
  return (
    <EmptyState title="Open League of Legends">
      Expose connects on its own when the client starts.
      <Loader />
    </EmptyState>
  );
}
