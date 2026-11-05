import { describe, expect, it } from "vitest";
import { EDGE, lineLeft } from "./align";

describe("lineLeft", () => {
  it("centres the words on his stance when they fit", () => {
    expect(lineLeft(210, 120, 120, 420)).toBe(150);
    expect(lineLeft(120, 120, 120, 420)).toBe(60);
    expect(lineLeft(330, 120, 120, 420)).toBe(270);
  });

  it("moves them only as far as keeping them in the window needs", () => {
    expect(lineLeft(40, 200, 200, 420)).toBe(EDGE);
    expect(lineLeft(400, 200, 200, 420)).toBe(420 - EDGE - 200);
  });

  it("places wrapped words by their own width, not their wider box", () => {
    // Two balanced lines 160 px wide in a 300 px box: the words, not the box, sit
    // against the edge, so they stay as close to him as they can.
    const left = lineLeft(40, 160, 300, 420);
    expect(left + 300 / 2 - 160 / 2).toBe(EDGE);
    const right = lineLeft(400, 160, 300, 420);
    expect(right + 300 / 2 + 160 / 2).toBe(420 - EDGE);
    expect(lineLeft(210, 160, 300, 420)).toBe(60);
  });

  it("keeps the start of words wider than the window in view", () => {
    expect(lineLeft(210, 500, 500, 420)).toBe(EDGE);
  });
});
