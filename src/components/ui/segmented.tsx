import { useRef, type KeyboardEvent } from "react";

import { cn } from "@/lib/utils";

interface SegmentedOption<T extends string | number> {
  value: T;
  label: string;
}

interface SegmentedProps<T extends string | number> {
  value: T;
  options: readonly SegmentedOption<T>[];
  onChange: (value: T) => void;
  // name the group: aria-label, or aria-labelledby pointing at a visible label.
  "aria-label"?: string;
  "aria-labelledby"?: string;
  disabled?: boolean;
  className?: string;
}

// a small set of mutually exclusive choices shown side by side. it behaves as a
// radio group: tab lands on the selected option, arrow keys (and home and end)
// move the selection.
function Segmented<T extends string | number>({
  value,
  options,
  onChange,
  disabled = false,
  className,
  ...labelling
}: SegmentedProps<T>) {
  const buttons = useRef<(HTMLButtonElement | null)[]>([]);
  const selected = options.findIndex((option) => option.value === value);

  // moves the selection and the focus together, wrapping at both ends.
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
    <div
      role="radiogroup"
      aria-disabled={disabled || undefined}
      className={cn(
        "inline-flex h-7 shrink-0 items-stretch gap-0.5 rounded-control border border-border bg-canvas p-0.5",
        disabled && "opacity-40",
        className,
      )}
      {...labelling}
    >
      {options.map((option, index) => {
        const checked = index === selected;
        // with nothing selected yet, the first option takes the tab stop.
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
            className={cn(
              "min-w-10 rounded-sm px-2 text-sm font-medium whitespace-nowrap transition-colors focus-visible:outline-offset-0",
              checked
                ? "bg-surface-raised text-fg shadow-sm"
                : "text-fg-secondary hover:text-fg",
            )}
          >
            {option.label}
          </button>
        );
      })}
    </div>
  );
}

export { Segmented, type SegmentedOption };
