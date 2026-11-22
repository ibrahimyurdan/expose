import { useId } from "react";

import { CheckGlyph } from "@/components/art/glyphs";

interface CheckboxProps {
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
  label: string;
  // one line under the label that says what the setting does.
  hint?: string;
  disabled?: boolean;
}

// an on/off setting: a small gold box, the label beside it, a hint under the
// label. space and enter toggle it like any button.
function Checkbox({ checked, onCheckedChange, label, hint, disabled = false }: CheckboxProps) {
  const hintId = useId();
  return (
    <button
      type="button"
      role="checkbox"
      aria-checked={checked}
      aria-disabled={disabled || undefined}
      aria-describedby={hint ? hintId : undefined}
      className="check"
      onClick={() => {
        if (!disabled) {
          onCheckedChange(!checked);
        }
      }}
    >
      <span className="check-box">
        <CheckGlyph />
      </span>
      <span className="check-text">
        <span className="check-label">{label}</span>
        {hint && (
          <span id={hintId} className="type-body">
            {hint}
          </span>
        )}
      </span>
    </button>
  );
}

export { Checkbox };
