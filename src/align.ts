// Where a line of words sits across the quote window.

/** Least gap (CSS px) between the words and the window's side, which near a screen edge is the screen's. */
export const EDGE = 16;

/**
 * Left edge for a line box `box` px wide whose words, centred in it, are `text` px
 * wide (narrower than the box once they wrap), so that the words centre on `anchor`
 * (his stance, in px from the window's left) but stay inside a window `stage` px wide.
 * Near a screen edge the window is off-centre from him, so centring on the window
 * would carry the words away from him; this moves them only as far as staying on
 * screen requires.
 */
export function lineLeft(anchor: number, text: number, box: number, stage: number): number {
  const centre = Math.max(EDGE + text / 2, Math.min(anchor, stage - EDGE - text / 2));
  return centre - box / 2;
}
