import { describe, expect, it } from "vitest";
import { follow } from "./follow";

/** A query whose reply is held until `reply` is called, and an event source. */
function source() {
  let emit: (v: string) => void = () => {};
  let reply: (v: string) => void = () => {};
  let stopped = false;
  const listen = (cb: (v: string) => void) => {
    emit = cb;
    return Promise.resolve(() => {
      stopped = true;
    });
  };
  const query = () => new Promise<string>((r) => (reply = r));
  return { listen, query, emit: (v: string) => emit(v), reply: (v: string) => reply(v), stopped: () => stopped };
}

const settle = () => new Promise((r) => setTimeout(r, 0));

describe("follow", () => {
  it("applies the current value when no change has happened", async () => {
    const s = source();
    const seen: string[] = [];
    follow(s.listen, s.query, (v) => seen.push(v));
    await settle();
    s.reply("passive");
    await settle();
    expect(seen).toEqual(["passive"]);
  });

  it("keeps a change that arrives before an older reply", async () => {
    const s = source();
    const seen: string[] = [];
    follow(s.listen, s.query, (v) => seen.push(v));
    await settle();
    s.emit("summoned");
    s.reply("passive");
    await settle();
    expect(seen).toEqual(["summoned"]);
  });

  it("applies nothing once stopped, and stops listening", async () => {
    const s = source();
    const seen: string[] = [];
    const stop = follow(s.listen, s.query, (v) => seen.push(v));
    await settle();
    stop();
    s.emit("late");
    s.reply("late");
    await settle();
    expect(seen).toEqual([]);
    expect(s.stopped()).toBe(true);
  });
});
