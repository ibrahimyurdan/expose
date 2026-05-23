import { Switch } from "@/components/ui/switch";
import { useAutoAccept } from "@/hooks/useAutoAccept";

export function AutoAcceptBar() {
  const { enabled, toggle } = useAutoAccept();
  return (
    <footer className="flex items-center justify-between px-4 py-3">
      <div className="min-w-0">
        <p className="hex-label text-xs text-gold">Auto-accept</p>
        <p className="truncate text-[11px] text-muted-foreground">
          {enabled ? "Ready checks accepted automatically" : "Accept ready checks manually"}
        </p>
      </div>
      <Switch checked={enabled} onCheckedChange={toggle} aria-label="Toggle auto-accept" />
    </footer>
  );
}
