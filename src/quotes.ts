// The knight's few words. Local only; shown when the user asks, never on a timer.

export const QUOTES = {
  resolve: ["I stand. I face what comes.", "Let the challenge come.", "Courage is action in the presence of fear."],
  endurance: ["Battered, but unbroken.", "The road is difficult. I continue.", "I have endured before. I can endure again."],
  determination: ["Forward.", "The work before me is mine to undertake.", "I will meet this moment."],
  recovery: ["Even the warrior may rest.", "Rest is not surrender."],
  victory: ["Another battle behind me.", "I have gained ground."],
} as const;

export const ALL_QUOTES: readonly string[] = Object.values(QUOTES).flat();

/** How many recent quotes to avoid repeating, when the library allows. */
const MEMORY = 4;

/**
 * Pick a quote at random, never the one just shown and preferably none of the last
 * few. `recent` is oldest-first.
 */
export function pickQuote(recent: readonly string[], random: () => number = Math.random): string {
  const avoid = new Set(recent.slice(-MEMORY));
  let pool = ALL_QUOTES.filter((q) => !avoid.has(q));
  if (pool.length === 0) pool = ALL_QUOTES.filter((q) => q !== recent.at(-1));
  return pool[Math.floor(random() * pool.length) % pool.length];
}
