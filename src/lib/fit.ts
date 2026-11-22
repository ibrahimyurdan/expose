// a layout may round a width up by a fraction of a pixel; that much over
// still fits, since the overflow it clips cannot be seen.
const SLACK = 0.5;

// how many chips of the given widths fit in a line `room` wide, `gap` apart.
// when they do not all fit, a "+N" chip `more` wide follows the ones that do,
// so room is kept for it. the first chip always shows, cut short if it must
// be.
export function fitCount(
  widths: readonly number[],
  gap: number,
  more: number,
  room: number,
): number {
  const all = widths.reduce((sum, width) => sum + width, 0) + gap * Math.max(widths.length - 1, 0);
  if (all <= room + SLACK) {
    return widths.length;
  }
  let used = more;
  let count = 0;
  for (const width of widths) {
    if (used + gap + width > room + SLACK) {
      break;
    }
    used += gap + width;
    count += 1;
  }
  return Math.max(count, 1);
}
