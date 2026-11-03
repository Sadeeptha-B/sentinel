import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "@fontsource/eb-garamond/latin-500-italic.css";
import { Knight } from "./Knight";
import { Whisper } from "./Whisper";
import "./styles.css";

// Both windows load this page; the window label decides what it shows.
const isWhisper = getCurrentWindow().label === "whisper";
document.documentElement.dataset.window = isWhisper ? "whisper" : "knight";

createRoot(document.getElementById("root")!).render(
  <StrictMode>{isWhisper ? <Whisper /> : <Knight />}</StrictMode>,
);
