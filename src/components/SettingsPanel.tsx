import { Switch } from "@/components/ui/switch";
import { useSettings } from "@/hooks/useSettings";
import { useUpdate } from "@/hooks/useUpdate";
import { bridge } from "@/lib/bridge";
import { cn } from "@/lib/utils";
import type { ScoutProvider } from "@/types";

const PROVIDERS: { id: ScoutProvider; label: string }[] = [
  { id: "opgg", label: "OP.GG" },
  { id: "ugg", label: "U.GG" },
  { id: "deeplol", label: "DeepLoL" },
  { id: "tracker", label: "Tracker" },
];

export function SettingsPanel() {
  const { settings, setScoutProvider, setAutoOpen, setLaunchAtLogin } = useSettings();
  const update = useUpdate();

  return (
    <div className="flex flex-col gap-5 p-4">
      <div>
        <p className="hex-label text-xs text-gold">Scout site</p>
        <p className="mb-2 text-[11px] text-muted-foreground">used by the scout-all button</p>
        <div className="flex gap-1">
          {PROVIDERS.map((provider) => (
            <button
              key={provider.id}
              type="button"
              onClick={() => setScoutProvider(provider.id)}
              className={cn(
                "flex-1 rounded-sm border px-1 py-1 text-[11px] transition-colors",
                settings.scoutProvider === provider.id
                  ? "border-gold bg-secondary text-gold-bright"
                  : "border-gold-deep text-muted-foreground hover:text-gold",
              )}
            >
              {provider.label}
            </button>
          ))}
        </div>
      </div>

      <SettingToggle
        label="Auto-open scout"
        hint="open the multi-search when champ select starts"
        checked={settings.autoOpen}
        onChange={setAutoOpen}
      />
      <SettingToggle
        label="Launch at login"
        hint="start Expose when you sign in"
        checked={settings.launchAtLogin}
        onChange={setLaunchAtLogin}
      />

      <div className="border-t border-gold-deep pt-3">
        <p className="hex-label text-xs text-gold">About</p>
        <p className="text-[11px] text-muted-foreground">
          Expose{update?.current ? ` v${update.current}` : ""}
        </p>
        {update?.available ? (
          <button
            type="button"
            onClick={() => bridge.openExternal(update.url).catch(() => {})}
            className="mt-1 text-[11px] text-gold-bright underline-offset-2 hover:underline"
          >
            update available: v{update.latest} — download
          </button>
        ) : null}
      </div>
    </div>
  );
}

function SettingToggle({
  label,
  hint,
  checked,
  onChange,
}: {
  label: string;
  hint: string;
  checked: boolean;
  onChange: (value: boolean) => void;
}) {
  return (
    <div className="flex items-center justify-between gap-3">
      <div className="min-w-0">
        <p className="hex-label text-xs text-gold">{label}</p>
        <p className="truncate text-[11px] text-muted-foreground">{hint}</p>
      </div>
      <Switch checked={checked} onCheckedChange={onChange} aria-label={label} />
    </div>
  );
}
