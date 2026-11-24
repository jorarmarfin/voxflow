import { render, screen, waitFor } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("./lib/api", async (orig) => {
  const real = await orig<typeof import("./lib/api")>();
  return {
    ...real,
    api: {
      getState: vi.fn().mockResolvedValue("IDLE"),
      getStatus: vi.fn().mockResolvedValue({ audio_ok: true, model_installed: false }),
      dismiss: vi.fn(),
      copyText: vi.fn(),
      insertText: vi.fn(),
      getDiagnostics: vi.fn().mockResolvedValue({ session: "x11", clipboard_tool: "xclip", notifications: true, auto_insert: false, insert_tool: null, log_path: null }),
      toggleRecording: vi.fn(),
      cancel: vi.fn(),
      getSettings: vi.fn().mockResolvedValue({
        device_id: null, model: "base", language: "es", output_mode: "preview", start_minimized: false, hotkey: "Ctrl+Alt+V",
      }),
      saveSettings: vi.fn(),
      onState: vi.fn().mockResolvedValue(() => {}),
      onNotice: vi.fn().mockResolvedValue(() => {}),
      onTranscript: vi.fn().mockResolvedValue(() => {}),
    },
  };
});

import App from "./App";

test("muestra estado listo y deshabilita grabar sin modelo", async () => {
  render(<App />);
  expect(await screen.findByText("Listo")).toBeTruthy();
  await waitFor(() => {
    const btn = screen.getByRole("button", { name: "Hablar con VoxFlow" }) as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });
  expect(screen.getByText(/Falta el modelo/)).toBeTruthy();
});
