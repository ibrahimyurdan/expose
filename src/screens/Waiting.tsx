import { Unplug } from "lucide-react";

import { EmptyState } from "@/components/ui/empty-state";

// league is not running (or expose has not found it yet). nothing to do here:
// the supervisor keeps looking and the header pill changes once it connects.
export function Waiting() {
  return (
    <EmptyState icon={Unplug} title="Open League of Legends">
      Expose connects on its own when the client starts.
    </EmptyState>
  );
}
