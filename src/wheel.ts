// Ctrl+wheel resizing in whole pixel-scale steps.

/** Wheel travel (pixel-mode deltaY, about one mouse notch) per resize step. */
export const NOTCH = 100;
/** Pixels per line, for wheels that report in lines. */
export const LINE = 33;

/**
 * Fold a wheel delta into the running total. Returns whole steps to take (positive
 * grows: wheel up) and the remainder to carry, so trackpads' small deltas add up.
 */
export function wheelSteps(total: number, deltaY: number): [steps: number, rest: number] {
  const sum = total - deltaY;
  const steps = Math.trunc(sum / NOTCH);
  return [steps, sum - steps * NOTCH];
}
