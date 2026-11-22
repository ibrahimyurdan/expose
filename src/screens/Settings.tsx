import { useId, type ReactNode } from "react";

import { FontCredits } from "@/components/shell/FontCredits";
import { RiotNotice } from "@/components/shell/RiotNotice";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { RadioGroup, type RadioOption } from "@/components/ui/radio-group";
import { Select, type SelectOption } from "@/components/ui/select";
import { usePushNotice } from "@/hooks/notices";
import type { useSettings } from "@/hooks/useSettings";
import { bridge } from "@/lib/bridge";
import { formatSeconds } from "@/lib/format";
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

// the ids a choice row hands its control, so the control is named by the
// row's visible label and described by its hint.
interface RowLabels {
  "aria-labelledby": string;
  "aria-describedby": string;
}

// the client's settings page: a breadcrumb title over a plain list, with no
// cards. on/off settings are checkboxes, choices sit under their label, and
// actions put a thin button at the row's end.
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
    <div className="settings">
      <div className="settings-title">
        <h1 className="type-h4">
          <span className="crumb">Expose</span>
          <span aria-hidden className="crumb-break">
            /
          </span>
          <span className="crumb-current">Settings</span>
        </h1>
      </div>

      <div className="settings-options">
        <Section title="General">
          <div className="setting">
            <Checkbox
              checked={settings.launchAtLogin}
              onCheckedChange={setLaunchAtLogin}
              label="Launch at login"
              hint="Start Expose when you sign in to your computer."
            />
          </div>
          <div className="setting">
            <Checkbox
              checked={autoAccept}
              onCheckedChange={onAutoAcceptChange}
              label="Auto-accept"
              hint="Accept the ready check as soon as a match is found."
            />
          </div>
        </Section>

        <Section title="Champ select">
          <div className="setting">
            <Checkbox
              checked={settings.revealRanked}
              onCheckedChange={controls.setRevealRanked}
              label="Reveal and match ranked teammates"
              hint="Show who is behind each alias in ranked solo/duo."
            />
          </div>
          <div className="setting">
            <Checkbox
              checked={settings.autoOpen}
              onCheckedChange={controls.setAutoOpen}
              label="Open scout site automatically"
              hint="Open your team on the scout site when champ select starts."
            />
          </div>
          <Choice label="Scout site" hint="Used for profile links and for scouting your team.">
            {(labels) => (
              <RadioGroup
                value={settings.scoutProvider}
                options={PROVIDERS}
                onChange={controls.setScoutProvider}
                {...labels}
              />
            )}
          </Choice>
        </Section>

        <Section title="Dodge">
          <Choice
            label="Last-second lead time"
            hint="An armed last-second dodge leaves the lobby with this much time left."
          >
            {(labels) => (
              <Select
                value={settings.dodgeLeadMs}
                options={LEAD_TIMES}
                onChange={controls.setDodgeLead}
                aria-labelledby={labels["aria-labelledby"]}
              />
            )}
          </Choice>
        </Section>

        <Section title="About">
          <Action
            label={update?.current ? `Expose ${update.current}` : "Expose"}
            hint={updateHint(update)}
          >
            {update?.available ? (
              <Button
                variant="thin"
                external
                onClick={() => bridge.openExternal(update.url).catch(() => {})}
              >
                Get {update.latest}
              </Button>
            ) : null}
          </Action>
          <Action label="Logs" hint="Useful when you report a problem.">
            <Button variant="thin" onClick={openLogsFolder}>
              Open logs folder
            </Button>
          </Action>
          <div className="legal">
            <RiotNotice className="type-body" />
            <FontCredits />
          </div>
        </Section>
      </div>
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
    <section aria-labelledby={id} className="settings-section">
      <h2 id={id} className="section-title type-h5">
        {title}
      </h2>
      {children}
    </section>
  );
}

// a setting with a choice: its label and hint, then the control under them.
function Choice({
  label,
  hint,
  children,
}: {
  label: string;
  hint: string;
  children: (labels: RowLabels) => ReactNode;
}) {
  const id = useId();
  const labels: RowLabels = {
    "aria-labelledby": `${id}-label`,
    "aria-describedby": `${id}-hint`,
  };
  return (
    <div className="setting">
      <p id={labels["aria-labelledby"]} className="setting-label">
        {label}
      </p>
      <p id={labels["aria-describedby"]} className="type-body">
        {hint}
      </p>
      {children(labels)}
    </div>
  );
}

// a row with an action: its label and hint, and a button at the end.
function Action({ label, hint, children }: { label: string; hint: string; children: ReactNode }) {
  return (
    <div className="setting setting--action">
      <div>
        <p className="setting-label">{label}</p>
        <p className="type-body">{hint}</p>
      </div>
      {children}
    </div>
  );
}
