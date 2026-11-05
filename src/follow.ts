// Following a value that Rust reports both on request and as an event on each change.

/**
 * Listen for changes, then ask for the current value, and pass each to `apply`.
 * The reply can arrive after a newer event (Ctrl+Alt+K at launch ending Passive), so
 * it's used only if no event came first; every change sends an event, so nothing is
 * missed. Returns a function that stops following; nothing is applied after it.
 */
export function follow<T>(
  listen: (cb: (value: T) => void) => Promise<() => void>,
  query: () => Promise<T>,
  apply: (value: T) => void,
): () => void {
  let live = true;
  let heard = false;
  const unlisten = listen((value) => {
    heard = true;
    if (live) apply(value);
  });
  void unlisten
    .then(() => query())
    .then((value) => {
      if (live && !heard) apply(value);
    });
  return () => {
    live = false;
    void unlisten.then((stop) => stop());
  };
}
