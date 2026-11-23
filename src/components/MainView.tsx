import { useState } from "react";
import { api, errMsg, STATE_LABEL, type RecState } from "../lib/api";

interface Props {
  state: RecState;
  engine: boolean;
  onNotice: (m: string | null) => void;
}

export default function MainView({ state, engine, onNotice }: Props) {
  const [text, setText] = useState("");
  const recording = state === "RECORDING";
  const busy = state === "TRANSCRIBING";

  const toggle = () => api.toggleRecording().catch((e) => onNotice(errMsg(e)));
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      onNotice("Copiado al portapapeles. Pega con Ctrl+V.");
    } catch (e) {
      onNotice(`No se pudo copiar: ${errMsg(e)}`);
    }
  };

  return (
    <main>
      <section className="card status" aria-live="polite">
        <span className={`dot ${state.toLowerCase()}`} aria-hidden="true" />
        <strong>{STATE_LABEL[state]}</strong>
        <span className="muted">{recording ? "Micrófono activo" : "Micrófono inactivo"}</span>
      </section>

      {!engine && (
        <p className="muted small">
          Esqueleto de la Fase 1: la captura de audio y la transcripción llegan en la Fase 2.
        </p>
      )}

      <button className="cta" onClick={toggle} disabled={busy || !engine}>
        {recording ? "Detener" : "Hablar con VoxFlow"}
      </button>
      {(recording || busy) && (
        <button className="secondary" onClick={() => api.cancel()}>Cancelar</button>
      )}

      <section className="card">
        <label htmlFor="preview">Vista previa (editable)</label>
        <textarea
          id="preview"
          value={text}
          onChange={(e) => setText(e.target.value)}
          placeholder="Aquí aparecerá la transcripción."
          rows={7}
        />
        <div className="actions">
          <button onClick={copy} disabled={!text}>Copiar</button>
          <button disabled title="La inserción automática no está disponible; usa Copiar y pega manualmente.">
            Insertar
          </button>
          <button onClick={() => setText("")} disabled={!text}>Descartar</button>
        </div>
      </section>
    </main>
  );
}
