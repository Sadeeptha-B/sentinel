import { useEffect, useRef, useState, type PointerEvent } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import meta from "./assets/knight.meta.json";
import full from "./assets/knight@full.webp";
import half from "./assets/knight@half.webp";
import * as ipc from "./ipc";

/** Window height below the figure reserved for his shadow. Matches SHADOW_BAND in knight.rs. */
const SHADOW_BAND = 0.03;
/** Pointer travel (px) that turns a press into a drag rather than a click. */
const DRAG_THRESHOLD = 4;
const RESIZE_STEP = 1.06;
/** Alpha (0–255) above which a pixel counts as part of him. */
const SOLID = 128;

const shadow = {
  left: `${((meta.feetLeft + meta.feetRight) / 2) * 100}%`,
  top: `${(meta.groundY - 0.008) * 100}%`,
  width: `${(meta.feetRight - meta.feetLeft) * 118}%`,
  height: `${SHADOW_BAND * 2.2 * 100}%`,
};

/** A small alpha map of the figure so clicks on empty pixels are ignored. */
function alphaMap(img: HTMLImageElement) {
  const w = Math.ceil(img.naturalWidth / 4);
  const h = Math.ceil(img.naturalHeight / 4);
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

  // Ctrl+wheel and Ctrl +/- resize; coalesced to one request per frame.
  useEffect(() => {
    let factor = 1;
    let frame = 0;
    const nudge = (f: number) => {
      if (locked.current) return;
      factor *= f;
      frame ||= requestAnimationFrame(() => {
        frame = 0;
        void ipc.resizeBy(factor);
        factor = 1;
      });
    };
    const onWheel = (e: WheelEvent) => {
      if (!e.ctrlKey) return;
      e.preventDefault();
      nudge(e.deltaY < 0 ? RESIZE_STEP : 1 / RESIZE_STEP);
    };
    const onKey = (e: KeyboardEvent) => {
      if (!e.ctrlKey) return;
      if (e.key === "+" || e.key === "=") nudge(RESIZE_STEP);
      else if (e.key === "-") nudge(1 / RESIZE_STEP);
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
    if (!locked.current) void getCurrentWindow().startDragging();
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
      <div className={ready ? "figure ready" : "figure"} style={{ aspectRatio: `${meta.width} / ${meta.height}` }}>
        <div className="shadow" style={shadow} />
        <img
          ref={imgRef}
          className="knight"
          src={full}
          srcSet={`${half} ${Math.floor(meta.width / 2)}w, ${full} ${meta.width}w`}
          sizes="100vw"
          alt="A battle-worn knight standing watch"
          draggable={false}
          onLoad={onLoad}
        />
      </div>
    </div>
  );
}
