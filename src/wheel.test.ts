import { describe, expect, it } from "vitest";
import { NOTCH, wheelSteps } from "./wheel";

describe("wheelSteps", () => {
  it("takes one step per notch, up grows and down shrinks", () => {
    expect(wheelSteps(0, -NOTCH)).toEqual([1, 0]);
    expect(wheelSteps(0, NOTCH)).toEqual([-1, 0]);
    expect(wheelSteps(0, -3 * NOTCH)).toEqual([3, 0]);
  });

  it("accumulates small trackpad deltas before stepping", () => {
    let total = 0;
    let steps = 0;
    for (let i = 0; i < 9; i++) {
      const [n, rest] = wheelSteps(total, -12);
      steps += n;
      total = rest;
    }
    expect(steps).toBe(1);
    expect(total).toBe(8);
  });

  it("lets a reversal cancel the carried remainder", () => {
    const [n1, rest] = wheelSteps(0, -60);
    const [n2, rest2] = wheelSteps(rest, 60);
    expect([n1, n2, rest2]).toEqual([0, 0, 0]);
  });
});
