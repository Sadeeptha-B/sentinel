// Thin typed wrappers over the Rust commands and events (see src-tauri/src/lib.rs).
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type UiState = { locked: boolean; passive: boolean; standWatch: boolean };
export type Side = "left" | "right";
/** `text: null` means "pick a quote yourself". */
export type WhisperRequest = { seq: number; text: string | null; side: Side; fontPx: number };

export const knightReady = () => invoke<void>("knight_ready");
export const uiState = () => invoke<UiState>("ui_state");
export const requestQuote = () => invoke<void>("request_quote");
export const resizeBy = (factor: number) => invoke<void>("resize_by", { factor });
export const openContextMenu = () => invoke<void>("open_context_menu");
export const whisperDone = (seq: number) => invoke<void>("whisper_done", { seq });

export const onUiState = (cb: (s: UiState) => void): Promise<UnlistenFn> =>
  listen<UiState>("sentinel://state", (e) => cb(e.payload));
export const onWhisper = (cb: (w: WhisperRequest) => void): Promise<UnlistenFn> =>
  listen<WhisperRequest>("sentinel://whisper", (e) => cb(e.payload));
