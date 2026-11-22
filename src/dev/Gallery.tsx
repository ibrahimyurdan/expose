// every expose drawing on one page, for checking them by eye in the browser
// preview (?screen=gallery). never part of the app: App loads it lazily, and
// only in the preview.

import { useState, type ReactNode } from "react";

import lipBottom from "@/assets/art/lip-bottom.svg";
import lipTop from "@/assets/art/lip-top.svg";
import rowDiamond from "@/assets/art/row-diamond.svg";
import { DodgeShape } from "@/components/art/DodgeShape";
import {
  CheckGlyph,
  ClockGlyph,
  CloseGlyph,
  ExactGlyph,
  ExternalGlyph,
  FineGlyph,
  GearGlyph,
  GripGlyph,
  MinimizeGlyph,
  RadioGlyph,
  SearchGlyph,
  UpDownGlyph,
  WarnGlyph,
} from "@/components/art/glyphs";
import { Mark } from "@/components/art/Mark";
import { MetalWork } from "@/components/art/MetalWork";
import { Ornament } from "@/components/art/Ornament";
import { PositionGlyph } from "@/components/art/PositionGlyph";
import { RankCrest } from "@/components/art/RankCrest";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Chip, Tag } from "@/components/ui/chip";
import { EmptyState } from "@/components/ui/empty-state";
import { Frame, Panel, Raised } from "@/components/ui/frame";
import { IconButton } from "@/components/ui/icon-button";
import { RadioGroup } from "@/components/ui/radio-group";
import { Loader } from "@/components/ui/loader";
import { Select } from "@/components/ui/select";
import { Status } from "@/components/ui/status";
import { Switch } from "@/components/ui/switch";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";

const GLYPHS = [
  ["minimize", MinimizeGlyph],
  ["gear", GearGlyph],
  ["close", CloseGlyph],
  ["grip", GripGlyph],
  ["warn", WarnGlyph],
  ["fine", FineGlyph],
  ["exact", ExactGlyph],
  ["external", ExternalGlyph],
  ["search", SearchGlyph],
  ["clock", ClockGlyph],
  ["updown", UpDownGlyph],
  ["check", CheckGlyph],
] as const;

// idle, hover and pressed, as the window controls use them.
const STATES = ["#a09b8c", "#f0e6d2", "#5b5a56"];

const POSITIONS = ["top", "jungle", "middle", "bottom", "utility", "fill"] as const;

const TIERS = [
  "iron",
  "bronze",
  "silver",
  "gold",
  "platinum",
  "emerald",
  "diamond",
  "master",
  "grandmaster",
  "challenger",
];

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="flex flex-col gap-3">
      <h2 className="type-h6">{title}</h2>
      <div className="flex flex-wrap items-end gap-4">{children}</div>
    </section>
  );
}

function Item({ label, children }: { label: string; children: ReactNode }) {
  return (
    <figure className="m-0 flex flex-col items-center gap-1.5">
      <div className="flex items-center gap-2">{children}</div>
      <figcaption className="type-tiny">{label}</figcaption>
    </figure>
  );
}

// every control in its resting, chosen and disabled states. hover, press and
// focus show by pointing and tabbing.
function Controls() {
  const [checked, setChecked] = useState(true);
  const [on, setOn] = useState(true);
  const [armed, setArmed] = useState(false);
  const [site, setSite] = useState<"opgg" | "ugg">("opgg");
  const [lead, setLead] = useState(1500);
  return (
    <>
      <Section title="Buttons">
        <Button>Framed</Button>
        <Button variant="primary">Get started</Button>
        <Button variant="thin">Open logs folder</Button>
        <Button variant="thin" external>
          View on OP.GG
        </Button>
        <Button variant="text">Text button</Button>
        <Button disabled>Disabled</Button>
        <Button variant="thin" disabled>
          Disabled
        </Button>
      </Section>
      <Section title="Icon buttons">
        <IconButton label="Minimize">
          <MinimizeGlyph />
        </IconButton>
        <IconButton label="Settings" aria-pressed>
          <GearGlyph />
        </IconButton>
        <IconButton label="Close">
          <CloseGlyph />
        </IconButton>
        <IconButton label="Copy" size="sm">
          <ExternalGlyph />
        </IconButton>
        <IconButton label="Scout your team" variant="round">
          <SearchGlyph />
        </IconButton>
        <IconButton label="Scout unavailable" variant="round" disabled>
          <SearchGlyph />
        </IconButton>
      </Section>
      <Section title="Checkboxes">
        <Checkbox
          checked={checked}
          onCheckedChange={setChecked}
          label="Auto-accept"
          hint="Accept the ready check as soon as a match is found."
        />
        <Checkbox checked={false} onCheckedChange={() => {}} label="Launch at login" />
        <Checkbox checked onCheckedChange={() => {}} label="Disabled" disabled />
      </Section>
      <Section title="Switches">
        <Switch checked={on} onCheckedChange={setOn} aria-label="Auto-accept" />
        <Switch
          danger
          checked={armed}
          onCheckedChange={setArmed}
          glyph={<ClockGlyph />}
          aria-label="Dodge at the last second"
        />
        <Switch checked={false} disabled aria-label="Disabled" />
      </Section>
      <Section title="Radios and dropdown">
        <RadioGroup
          value={site}
          options={[
            { value: "opgg", label: "OP.GG" },
            { value: "ugg", label: "U.GG" },
          ]}
          onChange={setSite}
          aria-label="Scout site"
        />
        <Select
          value={lead}
          options={[1000, 1500, 2000, 3000].map((ms) => ({ value: ms, label: `${ms / 1000} s` }))}
          onChange={setLead}
          aria-label="Last-second lead time"
        />
      </Section>
    </>
  );
}

// chips, tags, the status, frames, dividers, a tooltip, loading marks and an
// empty state.
function Pieces() {
  return (
    <>
      <Section title="Chips and tags">
        <Chip tone="red" warn>
          Autofilled, mains mid
        </Chip>
        <Chip tone="amber" warn>
          38% win rate, last 16
        </Chip>
        <Chip tone="neutral">Name One#NA1</Chip>
        <Chip tone="info">Info</Chip>
        <Tag tone="gold">Likely</Tag>
        <Tag tone="amber">Likely</Tag>
        <Tag tone="teal">Teal</Tag>
        <span className="count">3</span>
      </Section>
      <Section title="Status">
        <Status state="waiting" />
        <Status state="connected" />
        <Status state="champSelect" />
      </Section>
      <Section title="Frames">
        <Frame lips className="w-52 p-4">
          <p className="type-h5">Framed, with trim</p>
          <p className="type-body">Bright at the bottom.</p>
        </Frame>
        <Panel className="w-44 p-3">
          <p className="type-h6">Panel</p>
          <p className="type-body">A quiet container.</p>
        </Panel>
        <Raised className="w-44 p-3">
          <p className="type-micro">Raised</p>
          <div className="rule--fade my-2" />
          <p className="type-value">The details panel.</p>
        </Raised>
      </Section>
      <Section title="Tooltip, loading">
        <Tooltip>
          <TooltipTrigger asChild>
            <button type="button" className="btn-thin">
              Hover me
            </button>
          </TooltipTrigger>
          <TooltipContent>Version 0.2.1 is available</TooltipContent>
        </Tooltip>
        <span className="type-body flex items-center">
          Still checking 1 teammate
          <Loader />
        </span>
        <span className="skeleton h-3 w-20" />
        <span className="skeleton h-5 w-24" style={{ borderRadius: 16 }} />
      </Section>
      <Section title="Empty state">
        <div className="flex h-72 w-96 border border-hairline">
          <EmptyState title="Open League of Legends">
            Expose connects on its own when the client starts.
          </EmptyState>
        </div>
      </Section>
    </>
  );
}

export default function Gallery() {
  return (
    <TooltipProvider>
      <main className="flex h-full flex-col gap-6 overflow-y-auto p-4">
        <Controls />
        <Pieces />
        <Section title="Mark">
          <Item label="40">
            <Mark size={40} />
          </Item>
          <Item label="18">
            <Mark size={18} />
          </Item>
          <Item label="12, flat">
            <Mark size={12} />
          </Item>
        </Section>

        <Section title="Glyphs, 1x and 2x, idle hover pressed">
          {GLYPHS.map(([name, Glyph]) => (
            <Item key={name} label={name}>
              {STATES.map((color) => (
                <span key={color} className="flex items-end gap-1" style={{ color }}>
                  <Glyph width={18} height={18} />
                  <Glyph width={36} height={36} />
                </span>
              ))}
            </Item>
          ))}
        </Section>

        <Section title="Radios">
          <Item label="off">
            <RadioGlyph checked={false} width={18} height={18} />
            <RadioGlyph checked={false} width={36} height={36} />
          </Item>
          <Item label="on">
            <RadioGlyph checked width={18} height={18} />
            <RadioGlyph checked width={36} height={36} />
          </Item>
        </Section>

        <Section title="Metal-work and dodge">
          <Item label="left · right">
            <MetalWork side="left" />
            <MetalWork side="right" />
          </Item>
          <Item label="dodge shape">
            <DodgeShape />
          </Item>
        </Section>

        <Section title="Welcome ornament">
          <Item label="under the title">
            <Ornament />
          </Item>
        </Section>

        <Section title="Positions, 14 and 28">
          {POSITIONS.map((position) => (
            <Item key={position} label={position}>
              <span className="flex items-end gap-2 text-gold-3">
                <PositionGlyph position={position} className="size-3.5" />
                <PositionGlyph position={position} className="size-7" />
              </span>
            </Item>
          ))}
        </Section>

        <Section title="Rank crests, 14 and 28">
          {TIERS.map((tier) => (
            <Item key={tier} label={tier}>
              <RankCrest tier={tier} className="size-3.5" />
              <RankCrest tier={tier} className="size-7" />
            </Item>
          ))}
        </Section>

        <Section title="Frame trim and row divider tiles, 4x">
          <Item label="lip top">
            <img src={lipTop} width={48} height={16} alt="" style={{ imageRendering: "pixelated" }} />
          </Item>
          <Item label="lip bottom">
            <img src={lipBottom} width={48} height={16} alt="" style={{ imageRendering: "pixelated" }} />
          </Item>
          <Item label="row diamond">
            <img src={rowDiamond} width={36} height={36} alt="" style={{ imageRendering: "pixelated" }} />
          </Item>
        </Section>
      </main>
    </TooltipProvider>
  );
}
