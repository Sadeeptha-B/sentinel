import { describe, expect, it } from "vitest";
import { ALL_QUOTES, pickQuote } from "./quotes";

describe("pickQuote", () => {
  it("never repeats the quote just shown", () => {
    const recent: string[] = [];
    for (let i = 0; i < 500; i++) {
      const q = pickQuote(recent);
      expect(q).not.toBe(recent.at(-1));
      recent.push(q);
    }
  });

  it("avoids the last few quotes", () => {
    const recent = ALL_QUOTES.slice(0, 4);
    for (let i = 0; i < 200; i++) expect(recent).not.toContain(pickQuote(recent));
  });

  it("reaches every quote", () => {
    const seen = new Set<string>();
    const recent: string[] = [];
    for (let i = 0; i < 2000; i++) {
      const q = pickQuote(recent);
      seen.add(q);
      recent.push(q);
    }
    expect(seen.size).toBe(ALL_QUOTES.length);
  });

  it("stays in range at the random edge", () => {
    expect(ALL_QUOTES).toContain(pickQuote([], () => 0.999999999));
    expect(ALL_QUOTES).toContain(pickQuote([], () => 0));
  });
});
