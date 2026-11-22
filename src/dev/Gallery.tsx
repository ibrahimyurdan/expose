// every expose drawing on one page, for checking them by eye in the browser
// preview (?screen=gallery). never part of the app: App loads it lazily, and
// only in the preview.

import type { ReactNode } from "react";

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

export default function Gallery() {
  return (
    <main className="flex h-full flex-col gap-6 overflow-y-auto p-4">
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
  );
}
