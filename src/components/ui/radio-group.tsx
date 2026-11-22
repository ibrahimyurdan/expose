import { useRef, type KeyboardEvent } from "react";

import { RadioGlyph } from "@/components/art/glyphs";

export interface RadioOption<T extends string | number> {
  value: T;
  label: string;
}

interface RadioGroupProps<T extends string | number> {
  value: T;
  options: readonly RadioOption<T>[];
  onChange: (value: T) => void;
  // name the group: aria-label, or aria-labelledby pointing at a visible label.
  "aria-label"?: string;
  "aria-labelledby"?: string;
  "aria-describedby"?: string;
  disabled?: boolean;
}

// a row of diamond radios, for a choice between two or three values. tab
// lands on the chosen option; arrow keys (and home and end) move the choice.
function RadioGroup<T extends string | number>({
  value,
  options,
  onChange,
  disabled = false,
  ...labelling
}: RadioGroupProps<T>) {
  const buttons = useRef<(HTMLButtonElement | null)[]>([]);
  const selected = options.findIndex((option) => option.value === value);

  // moves the choice and the focus together, wrapping at both ends.
  const select = (index: number) => {
    const wrapped = (index + options.length) % options.length;
    const next = options[wrapped];
    if (!next) {
      return;
    }
    onChange(next.value);
    buttons.current[wrapped]?.focus();
  };

  const onKeyDown = (event: KeyboardEvent<HTMLButtonElement>, index: number) => {
    const moves: Record<string, number> = {
      ArrowRight: index + 1,
      ArrowDown: index + 1,
      ArrowLeft: index - 1,
      ArrowUp: index - 1,
      Home: 0,
      End: options.length - 1,
    };
    const target = moves[event.key];
    if (target !== undefined) {
      event.preventDefault();
      select(target);
    }
  };

  return (
    <div role="radiogroup" aria-disabled={disabled || undefined} className="radios" {...labelling}>
      {options.map((option, index) => {
        const checked = index === selected;
        // with nothing chosen yet, the first option takes the tab stop.
        const tabbable = checked || (selected === -1 && index === 0);
        return (
          <button
            key={String(option.value)}
            ref={(element) => {
              buttons.current[index] = element;
            }}
            type="button"
            role="radio"
            aria-checked={checked}
            tabIndex={tabbable ? 0 : -1}
            disabled={disabled}
            onClick={() => onChange(option.value)}
            onKeyDown={(event) => onKeyDown(event, index)}
            className="radio"
          >
            <RadioGlyph checked={checked} />
            {option.label}
          </button>
        );
      })}
    </div>
  );
}

export { RadioGroup };
