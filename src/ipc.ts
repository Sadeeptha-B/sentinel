// Thin typed wrappers over the Rust commands and events (see src-tauri/src/lib.rs).
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type UiState = { locked: boolean; passive: boolean; standWatch: boolean };
export type Placement = "below" | "over";
/** `text: null` means "pick a quote yourself". */
export type WhisperRequest = { seq: number; text: string | null; placement: Placement; fontPx: number };

export const knightReady = () => invoke<void>("knight_ready");
export const uiState = () => invoke<UiState>("ui_state");
export const requestQuote = () => invoke<void>("request_quote");
export const resizeStep = (steps: number) => invoke<void>("resize_step", { steps });
export const openContextMenu = () => invoke<void>("open_context_menu");
/** He was dragged while locked; Rust answers with a word from him. */
export const holdGround = () => invoke<void>("hold_ground");
export const whisperDone = (seq: number) => invoke<void>("whisper_done", { seq });

export const onUiState = (cb: (s: UiState) => void): Promise<UnlistenFn> =>
  listen<UiState>("sentinel://state", (e) => cb(e.payload));
export const onWhisper = (cb: (w: WhisperRequest) => void): Promise<UnlistenFn> =>
  listen<WhisperRequest>("sentinel://whisper", (e) => cb(e.payload));
