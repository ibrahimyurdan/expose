import { Switch } from "@/components/ui/switch";
import { useSettings } from "@/hooks/useSettings";
import { useUpdate } from "@/hooks/useUpdate";
import { bridge } from "@/lib/bridge";
import { cn } from "@/lib/utils";
import type { ScoutProvider } from "@/types";

const PROVIDERS: { id: ScoutProvider; label: string }[] = [
  { id: "opgg", label: "OP.GG" },
  { id: "ugg", label: "U.GG" },
];

const FOCUS_RING =
  "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background";

export function SettingsPanel() {
  const { settings, setScoutProvider, setAutoOpen, setRevealRanked, setLaunchAtLogin } =
    useSettings();
  const update = useUpdate();

  return (
    <div className="flex flex-col gap-5 px-3.5 py-4">
      <div>
        <p className="hex-label text-xs text-gold">Scout site</p>
        <p className="mb-2 text-[11px] text-muted-foreground">used by the scout-all button</p>
        <div className="flex gap-1.5">
          {PROVIDERS.map((provider) => (
            <button
              key={provider.id}
              type="button"
              onClick={() => setScoutProvider(provider.id)}
              aria-pressed={settings.scoutProvider === provider.id}
              className={cn(
                "flex-1 rounded-sm border px-1.5 py-1.5 text-[11px] transition-colors",
                FOCUS_RING,
                settings.scoutProvider === provider.id
                  ? "border-gold bg-secondary text-gold-bright"
                  : "border-gold-deep/70 text-muted-foreground hover:text-gold",
              )}
            >
              {provider.label}
            </button>
          ))}
        </div>
      </div>

      <SettingToggle
        label="Reveal in ranked"
        hint="show names Riot hides in anonymous champ select"
        checked={settings.revealRanked}
        onChange={setRevealRanked}
      />
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
            className={cn(
              "mt-1 rounded-sm text-[11px] text-gold-bright underline-offset-2 hover:underline",
              FOCUS_RING,
            )}
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
