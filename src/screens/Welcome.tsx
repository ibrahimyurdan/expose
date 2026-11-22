import { useId, type ComponentType, type SVGProps } from "react";

import { FineGlyph, SearchGlyph, WarnGlyph } from "@/components/art/glyphs";
import { Mark } from "@/components/art/Mark";
import { Ornament } from "@/components/art/Ornament";
import { RiotNotice } from "@/components/shell/RiotNotice";
import { Button } from "@/components/ui/button";
import { Frame } from "@/components/ui/frame";

const POINTS: { Glyph: ComponentType<SVGProps<SVGSVGElement>>; title: string; text: string }[] = [
  {
    Glyph: SearchGlyph,
    title: "See who you're queued with",
    text: "Rank, recent games and red flags for every teammate, even when ranked hides their names.",
  },
  {
    Glyph: FineGlyph,
    title: "Auto-accept ready checks",
    text: "Step away while you queue. Expose accepts the match for you.",
  },
  {
    Glyph: WarnGlyph,
    title: "Dodge now or at the last second",
    text: "Leave a lobby with two clicks, or arm a dodge that fires just before it locks in.",
  },
];

// the first-run screen: a framed card with what expose does, in three lines,
// and one way forward, on the card's bottom edge. it shows until "get
// started" is pressed once. riot's notice sits under the card.
export function Welcome({ onStart }: { onStart: () => void }) {
  const titleId = useId();
  return (
    <div className="welcome">
      <Frame lips role="region" aria-labelledby={titleId} className="welcome-card">
        <Mark size={40} />
        <h1 id={titleId} className="welcome-title type-h3">
          Welcome to Expose
        </h1>
        <Ornament className="welcome-ornament" />
        <p className="welcome-subtitle type-label">
          Your champ select companion for League of Legends.
        </p>

        <ul className="welcome-points">
          {POINTS.map(({ Glyph, title, text }) => (
            <li key={title}>
              <Glyph className="welcome-glyph" />
              <div>
                <p className="type-h5">{title}</p>
                <p className="type-body-m">{text}</p>
              </div>
            </li>
          ))}
        </ul>

        <div className="edge-actions">
          <div className="edge-actions-group">
            <Button variant="primary" onClick={onStart}>
              Get started
            </Button>
          </div>
        </div>
      </Frame>

      <RiotNotice className="welcome-notice type-tiny" />
    </div>
  );
}
