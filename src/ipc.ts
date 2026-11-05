// Thin typed wrappers over the Rust commands and events (see src-tauri/src/lib.rs).
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type UiState = { locked: boolean; passive: boolean; standWatch: boolean };
export type Placement = "below" | "over";
/** `text: null` means "pick a quote yourself". */
/** `stay: true` is the Stand Watch line: it holds until replaced instead of fading out. */
export type WhisperRequest = {
  seq: number;
  text: string | null;
  placement: Placement;
  /** Centre of his stance in px from the window's left; the words are set there. */
  anchorX: number;
  fontPx: number;
  stay: boolean;
};

export const knightReady = () => invoke<void>("knight_ready");
export const uiState = () => invoke<UiState>("ui_state");
export const requestQuote = () => invoke<void>("request_quote");
export const resizeStep = (steps: number) => invoke<void>("resize_step", { steps });
export const openContextMenu = () => invoke<void>("open_context_menu");
/** He was dragged while locked; Rust answers with a word from him. */
export const holdGround = () => invoke<void>("hold_ground");
/** The quote page is listening; until then Rust holds back the Stand Watch line. */
export const whisperReady = () => invoke<void>("whisper_ready");
export const whisperDone = (seq: number) => invoke<void>("whisper_done", { seq });

export const onUiState = (cb: (s: UiState) => void): Promise<UnlistenFn> =>
  listen<UiState>("sentinel://state", (e) => cb(e.payload));
export const onWhisper = (cb: (w: WhisperRequest) => void): Promise<UnlistenFn> =>
  listen<WhisperRequest>("sentinel://whisper", (e) => cb(e.payload));
/** The quote window was hidden; drop whatever it was showing. */
export const onWhisperClear = (cb: () => void): Promise<UnlistenFn> =>
  listen("sentinel://whisper-clear", () => cb());
