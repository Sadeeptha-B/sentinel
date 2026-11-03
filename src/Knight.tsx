import { useEffect, useRef, useState, type PointerEvent } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
// The sprite (with its baked contact shadow) is written by
// tools/asset-pipeline/pixel_knight.py --export.
import sprite from "./assets/knight.png";
import * as ipc from "./ipc";
import { LINE, wheelSteps } from "./wheel";

/** Pointer travel (px) that turns a press into a drag rather than a click. */
const DRAG_THRESHOLD = 4;
/** Alpha (0–255) above which a pixel counts as part of him (the shadow is fainter). */
const SOLID = 128;

/** The sprite's alpha, pixel for pixel, so clicks on empty or shadow pixels are ignored. */
function alphaMap(img: HTMLImageElement) {
  const w = img.naturalWidth;
  const h = img.naturalHeight;
  const ctx = Object.assign(document.createElement("canvas"), { width: w, height: h }).getContext("2d");
  if (!ctx) return null;
  ctx.drawImage(img, 0, 0, w, h);
  const data = ctx.getImageData(0, 0, w, h).data;
  return (fx: number, fy: number) => {
    if (fx < 0 || fy < 0 || fx >= 1 || fy >= 1) return false;
    return data[(Math.floor(fy * h) * w + Math.floor(fx * w)) * 4 + 3] > SOLID;
  };
}

export function Knight() {
  const [ready, setReady] = useState(false);
  const imgRef = useRef<HTMLImageElement>(null);
  const solidAt = useRef<((fx: number, fy: number) => boolean) | null>(null);
  const locked = useRef(false);
  const press = useRef<{ x: number; y: number } | null>(null);

  useEffect(() => {
    void ipc.uiState().then((s) => (locked.current = s.locked));
    const unlisten = ipc.onUiState((s) => (locked.current = s.locked));
    return () => void unlisten.then((f) => f());
  }, []);

  // Ctrl+wheel and Ctrl +/- resize in whole pixel-scale steps, at most one request
  // per frame. Rust refuses them while he's locked, and tells the user so.
  useEffect(() => {
    let wheel = 0;
    let pending = 0;
    let frame = 0;
    const step = (n: number) => {
      if (n === 0) return;
      pending += n;
      frame ||= requestAnimationFrame(() => {
        frame = 0;
        if (pending) void ipc.resizeStep(pending);
        pending = 0;
      });
    };
    const onWheel = (e: WheelEvent) => {
      if (!e.ctrlKey) return;
      e.preventDefault();
      const delta = e.deltaMode === WheelEvent.DOM_DELTA_LINE ? e.deltaY * LINE : e.deltaY;
      const [n, rest] = wheelSteps(wheel, delta);
      wheel = rest;
      step(n);
    };
    const onKey = (e: KeyboardEvent) => {
      if (!e.ctrlKey) return;
      if (e.key === "+" || e.key === "=") step(1);
      else if (e.key === "-") step(-1);
      else return;
      e.preventDefault();
    };
    window.addEventListener("wheel", onWheel, { passive: false });
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("wheel", onWheel);
      window.removeEventListener("keydown", onKey);
      cancelAnimationFrame(frame);
    };
  }, []);

  const onLoad = async () => {
    const img = imgRef.current;
    if (!img) return;
    await img.decode().catch(() => {});
    solidAt.current = alphaMap(img);
    await ipc.knightReady();
    requestAnimationFrame(() => setReady(true));
  };

  const onFigure = (e: PointerEvent) => {
    const img = imgRef.current;
    const solid = solidAt.current;
    if (!img || !solid) return true;
    const r = img.getBoundingClientRect();
    return solid((e.clientX - r.left) / r.width, (e.clientY - r.top) / r.height);
  };

  const onPointerDown = (e: PointerEvent) => {
    press.current = e.button === 0 && onFigure(e) ? { x: e.screenX, y: e.screenY } : null;
  };

  const onPointerMove = (e: PointerEvent) => {
    const p = press.current;
    if (!p || !(e.buttons & 1)) return;
    if (Math.hypot(e.screenX - p.x, e.screenY - p.y) < DRAG_THRESHOLD) return;
    press.current = null;
    if (locked.current) void ipc.holdGround();
    else void getCurrentWindow().startDragging();
  };

  const onPointerUp = (e: PointerEvent) => {
    if (e.button !== 0 || !press.current) return;
    press.current = null;
    void ipc.requestQuote();
  };

  return (
    <div
      className="stage"
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={() => (press.current = null)}
      onContextMenu={(e) => {
        e.preventDefault();
        if (onFigure(e as unknown as PointerEvent)) void ipc.openContextMenu();
      }}
    >
      <div className={ready ? "figure ready" : "figure"}>
        <img
          ref={imgRef}
          className="knight"
          src={sprite}
          alt="A battle-worn knight standing watch"
          draggable={false}
          onLoad={onLoad}
        />
      </div>
    </div>
  );
}
