import { useCallback, useEffect, useRef, useState } from "react";
import { pickQuote } from "./quotes";
import * as ipc from "./ipc";

const FADE_IN = 700;
const FADE_OUT = 1400;
const CROSSFADE = 350;
/** Long enough to read without hurrying, short enough not to linger. */
const holdFor = (text: string) => 3200 + 55 * text.length;

type Line = Omit<ipc.WhisperRequest, "text"> & { text: string; leaving: boolean };

/** The quote window: one line at a time, fading in beside him and away again. */
export function Whisper() {
  const [lines, setLines] = useState<Line[]>([]);
  const recent = useRef<string[]>([]);

  useEffect(() => {
    const unlisten = ipc.onWhisper((req) => {
      let text = req.text;
      if (text === null) {
        text = pickQuote(recent.current);
        recent.current = [...recent.current, text].slice(-6);
      }
      const line: Line = { ...req, text, leaving: false };
      setLines((prev) => [...prev.map((l) => ({ ...l, leaving: true })), line]);
    });
    const unlistenClear = ipc.onWhisperClear(() => setLines([]));
    // Under StrictMode the first setup is torn down at once; only the live one may
    // tell Rust that lines will be heard.
    let live = true;
    void Promise.all([unlisten, unlistenClear]).then(() => {
      if (live) void ipc.whisperReady();
    });
    return () => {
      live = false;
      void unlisten.then((f) => f());
      void unlistenClear.then((f) => f());
    };
  }, []);

  // Stable identity: lines' animation effects depend on it and must not restart.
  const gone = useCallback((seq: number) => setLines((prev) => prev.filter((l) => l.seq !== seq)), []);

  return (
    <div className="whisper-stage">
      {lines.map((l) => (
        <WhisperLine key={l.seq} line={l} onGone={gone} />
      ))}
    </div>
  );
}

function WhisperLine({ line, onGone }: { line: Line; onGone: (seq: number) => void }) {
  const ref = useRef<HTMLParagraphElement>(null);
  const main = useRef<Animation | null>(null);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    if (line.stay) {
      // The Stand Watch line: fade in and stay until a newer line replaces it.
      const anim = el.animate(
        [
          { opacity: 0, transform: "translateY(4px)" },
          { opacity: 1, transform: "translateY(0)" },
        ],
        { duration: FADE_IN, delay: CROSSFADE / 2, easing: "cubic-bezier(0.2, 0.6, 0.3, 1)", fill: "forwards" },
      );
      main.current = anim;
      return () => anim.cancel();
    }
    const hold = holdFor(line.text);
    const total = FADE_IN + hold + FADE_OUT;
    const anim = el.animate(
      [
        { opacity: 0, transform: "translateY(4px)", easing: "cubic-bezier(0.2, 0.6, 0.3, 1)" },
        { opacity: 1, transform: "translateY(0)", offset: FADE_IN / total },
        { opacity: 1, transform: "translateY(0)", offset: (FADE_IN + hold) / total, easing: "ease-in-out" },
        { opacity: 0, transform: "translateY(0)" },
      ],
      { duration: total, delay: CROSSFADE / 2, fill: "forwards" },
    );
    main.current = anim;
    anim.finished.then(
      () => {
        onGone(line.seq);
        void ipc.whisperDone(line.seq);
      },
      () => {}, // cancelled: superseded or unmounted
    );
    return () => anim.cancel();
  }, [line.seq, line.text, line.stay, onGone]);

  useEffect(() => {
    const el = ref.current;
    if (!line.leaving || !el) return;
    main.current?.commitStyles();
    main.current?.cancel();
    const out = el.animate([{ opacity: getComputedStyle(el).opacity }, { opacity: 0 }], {
      duration: CROSSFADE,
      easing: "ease-out",
      fill: "forwards",
    });
    out.finished.then(() => onGone(line.seq), () => {});
    return () => out.cancel();
  }, [line.leaving, line.seq, onGone]);

  return (
    <p ref={ref} className={`whisper ${line.placement}`} style={{ fontSize: `${line.fontPx}px` }}>
      {line.text}
    </p>
  );
}
