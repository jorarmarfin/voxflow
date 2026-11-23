import { render, screen, waitFor } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("./lib/api", async (orig) => {
  const real = await orig<typeof import("./lib/api")>();
  return {
    ...real,
    api: {
      getState: vi.fn().mockResolvedValue("IDLE"),
      engineAvailable: vi.fn().mockResolvedValue(false),
      toggleRecording: vi.fn(),
      cancel: vi.fn(),
      getSettings: vi.fn().mockResolvedValue({
        device_id: null, model: "base", language: "es", output_mode: "preview", start_minimized: false,
      }),
      saveSettings: vi.fn(),
      onState: vi.fn().mockResolvedValue(() => {}),
      onNotice: vi.fn().mockResolvedValue(() => {}),
    },
  };
});

import App from "./App";

test("muestra estado listo y deshabilita grabar sin motor", async () => {
  render(<App />);
  expect(await screen.findByText("Listo")).toBeTruthy();
  await waitFor(() => {
    const btn = screen.getByRole("button", { name: "Hablar con VoxFlow" }) as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });
  expect(screen.getByText(/Fase 2/)).toBeTruthy();
});
