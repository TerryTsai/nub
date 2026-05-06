import { Combobox } from "./Combobox";

interface Option<T extends string> {
  value: T;
  label: string;
}

/** Filter pill in the subnav. Single control per filterable axis — tap to
 * open a sheet, pick a value, dialog closes. Replaces the previous
 * segmented pill-tabs which scaled poorly past 2-3 values and didn't
 * compose with multiple axes on the same row.
 *
 * `attribute` is the axis name shown in muted tone before the value, so
 * the pill reads as e.g. `status: All ▾`. */
export function Filters<T extends string>({
  attribute,
  value,
  onChange,
  options,
}: {
  attribute: string;
  value: T;
  onChange: (v: T) => void;
  options: Option<T>[];
}) {
  return (
    <Combobox
      pill
      attribute={attribute}
      value={value}
      onChange={(v) => onChange(v as T)}
      options={options}
    />
  );
}
