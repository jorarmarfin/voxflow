import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type RecState = "IDLE" | "RECORDING" | "TRANSCRIBING" | "READY";
export type OutputMode = "preview" | "clipboard";

export interface Settings {
  device_id: string | null;
  model: string;
  language: string;
  output_mode: OutputMode;
  start_minimized: boolean;
}

export const api = {
  getState: () => invoke<RecState>("get_state"),
  engineAvailable: () => invoke<boolean>("engine_available"),
  toggleRecording: () => invoke<RecState>("toggle_recording"),
  cancel: () => invoke<RecState>("cancel"),
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),
  onState: (cb: (s: RecState) => void): Promise<UnlistenFn> =>
    listen<RecState>("state-changed", (e) => cb(e.payload)),
  onNotice: (cb: (msg: string) => void): Promise<UnlistenFn> =>
    listen<string>("notice", (e) => cb(e.payload)),
};

export const STATE_LABEL: Record<RecState, string> = {
  IDLE: "Listo",
  RECORDING: "Grabando…",
  TRANSCRIBING: "Transcribiendo…",
  READY: "Texto listo",
};

export function errMsg(e: unknown): string {
  return typeof e === "string" ? e : e instanceof Error ? e.message : "Error desconocido";
}
