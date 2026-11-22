import * as SelectPrimitive from "@radix-ui/react-select";

import { UpDownGlyph } from "@/components/art/glyphs";

export interface SelectOption<T extends string | number> {
  value: T;
  label: string;
}

interface SelectProps<T extends string | number> {
  value: T;
  options: readonly SelectOption<T>[];
  onChange: (value: T) => void;
  // name it: aria-label, or aria-labelledby pointing at a visible label.
  "aria-label"?: string;
  "aria-labelledby"?: string;
  disabled?: boolean;
}

// the framed dropdown, for a choice with more than two values. the open list
// sits right under the field, as wide as it.
function Select<T extends string | number>({
  value,
  options,
  onChange,
  disabled = false,
  ...labelling
}: SelectProps<T>) {
  return (
    <SelectPrimitive.Root
      value={String(value)}
      onValueChange={(chosen) => {
        const option = options.find((candidate) => String(candidate.value) === chosen);
        if (option) {
          onChange(option.value);
        }
      }}
      disabled={disabled}
    >
      <SelectPrimitive.Trigger className="select" {...labelling}>
        <SelectPrimitive.Value />
        <SelectPrimitive.Icon asChild>
          <UpDownGlyph />
        </SelectPrimitive.Icon>
      </SelectPrimitive.Trigger>
      <SelectPrimitive.Portal>
        <SelectPrimitive.Content className="select-list" position="popper" sideOffset={2}>
          <SelectPrimitive.Viewport>
            {options.map((option) => (
              <SelectPrimitive.Item
                key={String(option.value)}
                value={String(option.value)}
                className="select-option"
              >
                <SelectPrimitive.ItemText>{option.label}</SelectPrimitive.ItemText>
              </SelectPrimitive.Item>
            ))}
          </SelectPrimitive.Viewport>
        </SelectPrimitive.Content>
      </SelectPrimitive.Portal>
    </SelectPrimitive.Root>
  );
}

export { Select };
