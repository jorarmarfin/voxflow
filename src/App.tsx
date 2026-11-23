import { useCallback, useEffect, useState } from "react";
import { api, errMsg, type RecState, type Settings, type Status } from "./lib/api";
import MainView from "./components/MainView";
import SettingsView from "./components/SettingsView";

export default function App() {
  const [view, setView] = useState<"main" | "settings">("main");
  const [state, setState] = useState<RecState>("IDLE");
  const [status, setStatus] = useState<Status>({ audio_ok: false, model_installed: false });
  const [settings, setSettings] = useState<Settings | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [transcript, setTranscript] = useState<{ text: string; n: number } | null>(null);

  const refreshStatus = useCallback(() => {
    api.getStatus().then(setStatus).catch((e) => setNotice(errMsg(e)));
  }, []);

  useEffect(() => {
    api.getState().then(setState).catch((e) => setNotice(errMsg(e)));
    api.getSettings().then(setSettings).catch((e) => setNotice(errMsg(e)));
    refreshStatus();
    const subs = [
      api.onState(setState),
      api.onNotice(setNotice),
      api.onTranscript((text) => setTranscript((t) => ({ text, n: (t?.n ?? 0) + 1 }))),
    ];
    return () => {
      subs.forEach((p) => p.then((un) => un()));
    };
  }, [refreshStatus]);

  return (
    <div className="app">
      <header className="topbar">
        <img src="/logo.png" alt="" className="logo" />
        <h1>VoxFlow</h1>
        <nav>
          <button className={view === "main" ? "tab active" : "tab"} onClick={() => setView("main")}>Dictado</button>
          <button className={view === "settings" ? "tab active" : "tab"} onClick={() => setView("settings")}>Preferencias</button>
        </nav>
      </header>
      {notice && (
        <div className="notice" role="alert">
          <span>{notice}</span>
          <button aria-label="Cerrar aviso" onClick={() => setNotice(null)}>×</button>
        </div>
      )}
      {view === "main" ? (
        <MainView
          state={state}
          status={status}
          settings={settings}
          transcript={transcript}
          onNotice={setNotice}
          onGoSettings={() => setView("settings")}
        />
      ) : settings ? (
        <SettingsView settings={settings} onSaved={setSettings} onNotice={setNotice} onModelChange={refreshStatus} />
      ) : (
        <p className="muted">Cargando preferencias…</p>
      )}
    </div>
  );
}
