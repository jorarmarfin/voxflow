import { useState } from "react";
import { api, errMsg, type OutputMode, type Settings } from "../lib/api";

interface Props {
  settings: Settings;
  onSaved: (s: Settings) => void;
  onNotice: (m: string | null) => void;
}

export default function SettingsView({ settings, onSaved, onNotice }: Props) {
  const [draft, setDraft] = useState<Settings>(settings);
  const set = <K extends keyof Settings>(k: K, v: Settings[K]) => setDraft({ ...draft, [k]: v });

  const save = async () => {
    try {
      onSaved(await api.saveSettings(draft));
      onNotice("Preferencias guardadas.");
    } catch (e) {
      onNotice(errMsg(e));
    }
  };

  return (
    <main>
      <section className="card form">
        <label htmlFor="device">Micrófono</label>
        <select id="device" value={draft.device_id ?? ""} onChange={(e) => set("device_id", e.target.value || null)}>
          <option value="">Predeterminado del sistema</option>
        </select>

        <label htmlFor="model">Modelo</label>
        <select id="model" value={draft.model} onChange={(e) => set("model", e.target.value)}>
          <option value="base">base (~142 MB)</option>
        </select>

        <label htmlFor="lang">Idioma</label>
        <select id="lang" value={draft.language} onChange={(e) => set("language", e.target.value)}>
          <option value="es">Español</option>
          <option value="en">English</option>
        </select>

        <label htmlFor="out">Modo de salida</label>
        <select id="out" value={draft.output_mode} onChange={(e) => set("output_mode", e.target.value as OutputMode)}>
          <option value="preview">Vista previa antes de copiar</option>
          <option value="clipboard">Copiar directo al portapapeles</option>
        </select>

        <label className="check">
          <input type="checkbox" checked={draft.start_minimized} onChange={(e) => set("start_minimized", e.target.checked)} />
          Iniciar minimizado en la bandeja
        </label>

        <button className="cta" onClick={save}>Guardar</button>
      </section>
      <section className="card">
        <h2>Privacidad</h2>
        <p className="muted small">
          El audio y el texto se procesan solo en este equipo. Sin telemetría, sin historial y sin guardar audio.
        </p>
        <h2>Atajo de teclado</h2>
        <p className="muted small">
          En GNOME, asigna un atajo personalizado que ejecute <code>voxflow --toggle</code> (disponible en la Fase 3).
        </p>
      </section>
    </main>
  );
}
