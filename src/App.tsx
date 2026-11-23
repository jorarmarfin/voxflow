import { useEffect, useState } from "react";
import { api, errMsg, type RecState, type Settings } from "./lib/api";
import MainView from "./components/MainView";
import SettingsView from "./components/SettingsView";

export default function App() {
  const [view, setView] = useState<"main" | "settings">("main");
  const [state, setState] = useState<RecState>("IDLE");
  const [engine, setEngine] = useState(false);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    api.getState().then(setState).catch((e) => setNotice(errMsg(e)));
    api.engineAvailable().then(setEngine).catch(() => setEngine(false));
    api.getSettings().then(setSettings).catch((e) => setNotice(errMsg(e)));
    const subs = [api.onState(setState), api.onNotice(setNotice)];
    return () => {
      subs.forEach((p) => p.then((un) => un()));
    };
  }, []);

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
        <MainView state={state} engine={engine} onNotice={setNotice} />
      ) : settings ? (
        <SettingsView settings={settings} onSaved={setSettings} onNotice={setNotice} />
      ) : (
        <p className="muted">Cargando preferencias…</p>
      )}
    </div>
  );
}
