import { useId, type ReactNode } from "react";

import { Button } from "@/components/ui/button";
import { Panel } from "@/components/ui/frame";
import { RadioGroup, type RadioOption } from "@/components/ui/radio-group";
import { Select, type SelectOption } from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { usePushNotice } from "@/hooks/notices";
import type { useSettings } from "@/hooks/useSettings";
import { bridge } from "@/lib/bridge";
import { formatSeconds } from "@/lib/format";
import { cn } from "@/lib/utils";
import type { ScoutProvider, UpdateInfo } from "@/types";

const PROVIDERS: readonly RadioOption<ScoutProvider>[] = [
  { value: "opgg", label: "OP.GG" },
  { value: "ugg", label: "U.GG" },
];

// the backend accepts 1000 to 3000 ms; these are the steps worth offering.
const LEAD_TIMES: readonly SelectOption<number>[] = [1000, 1500, 2000, 3000].map((ms) => ({
  value: ms,
  label: formatSeconds(ms),
}));

// the ids a row hands its control, so the control is named by the row's
// visible label and described by its hint.
interface RowLabels {
  "aria-labelledby": string;
  "aria-describedby": string;
}

export function Settings({
  controls,
  autoAccept,
  onAutoAcceptChange,
  update,
}: {
  controls: ReturnType<typeof useSettings>;
  autoAccept: boolean;
  onAutoAcceptChange: (enabled: boolean) => void;
  update: UpdateInfo | null;
}) {
  const push = usePushNotice();
  const { settings } = controls;

  const setLaunchAtLogin = (enabled: boolean) => {
    controls
      .setLaunchAtLogin(enabled)
      .catch(() => push({ level: "error", message: "Couldn't change launch at login." }));
  };

  const openLogsFolder = () => {
    bridge
      .openLogsFolder()
      .catch(() => push({ level: "error", message: "Couldn't open the logs folder." }));
  };

  return (
    <div className="flex flex-col gap-6 p-4 pb-8">
      <h1 className="px-1 text-md font-semibold text-fg">Settings</h1>

      <Section title="General">
        <Row label="Launch at login" hint="Start Expose when you sign in to your computer.">
          {(labels) => (
            <Switch checked={settings.launchAtLogin} onCheckedChange={setLaunchAtLogin} {...labels} />
          )}
        </Row>
        <Row label="Auto-accept" hint="Accept the ready check as soon as a match is found.">
          {(labels) => (
            <Switch checked={autoAccept} onCheckedChange={onAutoAcceptChange} {...labels} />
          )}
        </Row>
      </Section>

      <Section title="Champ select">
        <Row
          label="Reveal and match ranked teammates"
          hint="Show who is behind each alias in ranked solo/duo."
        >
          {(labels) => (
            <Switch
              checked={settings.revealRanked}
              onCheckedChange={controls.setRevealRanked}
              {...labels}
            />
          )}
        </Row>
        <Row
          label="Open scout site automatically"
          hint="Open your team on the scout site when champ select starts."
        >
          {(labels) => (
            <Switch checked={settings.autoOpen} onCheckedChange={controls.setAutoOpen} {...labels} />
          )}
        </Row>
        <Row label="Scout site" hint="Used for profile links and for scouting your team.">
          {(labels) => (
            <RadioGroup
              value={settings.scoutProvider}
              options={PROVIDERS}
              onChange={controls.setScoutProvider}
              {...labels}
            />
          )}
        </Row>
      </Section>

      <Section title="Dodge">
        <Row
          label="Last-second lead time"
          hint="An armed last-second dodge leaves the lobby with this much time left."
          stacked
        >
          {(labels) => (
            <Select
              value={settings.dodgeLeadMs}
              options={LEAD_TIMES}
              onChange={controls.setDodgeLead}
              aria-labelledby={labels["aria-labelledby"]}
            />
          )}
        </Row>
      </Section>

      <Section title="About">
        <Row label={update?.current ? `Expose ${update.current}` : "Expose"} hint={updateHint(update)}>
          {() =>
            update?.available ? (
              <Button
                variant="thin"
                external
                onClick={() => bridge.openExternal(update.url).catch(() => {})}
              >
                Get {update.latest}
              </Button>
            ) : null
          }
        </Row>
        <Row label="Logs" hint="Useful when you report a problem.">
          {() => (
            <Button variant="thin" onClick={openLogsFolder}>
              Open logs folder
            </Button>
          )}
        </Row>
      </Section>
    </div>
  );
}

// the update check stores an empty url when github could not be reached.
function updateHint(update: UpdateInfo | null): string {
  if (!update) {
    return "Checking for updates…";
  }
  if (update.available) {
    return `Version ${update.latest} is available.`;
  }
  return update.url ? "You have the latest version." : "Couldn't check for updates.";
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  const id = useId();
  return (
    <section aria-labelledby={id} className="flex flex-col gap-2">
      <h2 id={id} className="px-1 text-sm font-medium text-fg-secondary">
        {title}
      </h2>
      <Panel className="divide-y divide-border">{children}</Panel>
    </section>
  );
}

// one setting: its label and hint on the left, the control on the right, or
// below them when the control is too wide to share the line (stacked).
function Row({
  label,
  hint,
  stacked = false,
  children,
}: {
  label: string;
  hint: string;
  stacked?: boolean;
  children: (labels: RowLabels) => ReactNode;
}) {
  const id = useId();
  const labels: RowLabels = {
    "aria-labelledby": `${id}-label`,
    "aria-describedby": `${id}-hint`,
  };
  return (
    <div className={cn("flex gap-4 px-4 py-3", stacked ? "flex-col gap-3" : "items-center")}>
      <div className="min-w-0 flex-1">
        <p id={labels["aria-labelledby"]} className="text-base text-fg">
          {label}
        </p>
        <p id={labels["aria-describedby"]} className="text-sm text-fg-secondary">
          {hint}
        </p>
      </div>
      {children(labels)}
    </div>
  );
}
