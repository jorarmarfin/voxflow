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

export interface Status {
  audio_ok: boolean;
  model_installed: boolean;
}
export interface Device {
  id: string;
  label: string;
}
export interface ModelStatus {
  id: string;
  installed: boolean;
  size_bytes: number;
}
export interface Progress {
  downloaded: number;
  total: number;
}
export interface DownloadDone {
  ok: boolean;
  error: string | null;
}

export const api = {
  getState: () => invoke<RecState>("get_state"),
  getStatus: () => invoke<Status>("get_status"),
  listDevices: () => invoke<Device[]>("list_devices"),
  toggleRecording: () => invoke<RecState>("toggle_recording"),
  cancel: () => invoke<RecState>("cancel"),
  dismiss: () => invoke<RecState>("dismiss"),
  modelStatus: () => invoke<ModelStatus[]>("model_status"),
  downloadModel: (id: string) => invoke<void>("download_model", { id }),
  cancelDownload: () => invoke<void>("cancel_download"),
  deleteModel: (id: string) => invoke<void>("delete_model", { id }),
  onTranscript: (cb: (t: string) => void): Promise<UnlistenFn> =>
    listen<string>("transcript", (e) => cb(e.payload)),
  onProgress: (cb: (p: Progress) => void): Promise<UnlistenFn> =>
    listen<Progress>("download-progress", (e) => cb(e.payload)),
  onDownloadDone: (cb: (d: DownloadDone) => void): Promise<UnlistenFn> =>
    listen<DownloadDone>("download-done", (e) => cb(e.payload)),
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
