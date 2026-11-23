import { useEffect, useState } from "react";
import { api, errMsg, STATE_LABEL, type RecState, type Status } from "../lib/api";

interface Props {
  state: RecState;
  status: Status;
  transcript: { text: string; n: number } | null;
  onNotice: (m: string | null) => void;
  onGoSettings: () => void;
}

export default function MainView({ state, status, transcript, onNotice, onGoSettings }: Props) {
  const [text, setText] = useState("");
  const recording = state === "RECORDING";
  const busy = state === "TRANSCRIBING";

  const copy = async (value: string) => {
    try {
      await api.copyText(value);
      onNotice("Copiado al portapapeles. Pega con Ctrl+V.");
    } catch (e) {
      onNotice(`No se pudo copiar: ${errMsg(e)}`);
    }
  };

  // Insertar: sin mecanismo seguro validado, el backend copia y avisamos de pegar a mano.
  const insert = async () => {
    try {
      await api.insertText(text);
      onNotice("Inserción automática no disponible en esta sesión: el texto está copiado, pégalo con Ctrl+V.");
    } catch (e) {
      onNotice(`No se pudo copiar: ${errMsg(e)}`);
    }
  };

  // Nueva transcripción: mostrarla (en modo "clipboard" el backend ya la copió).
  useEffect(() => {
    if (transcript) setText(transcript.text);
  }, [transcript]);

  const toggle = () => api.toggleRecording().catch((e) => onNotice(errMsg(e)));
  const discard = () => {
    setText("");
    void api.dismiss();
  };

  const blocker = !status.audio_ok
    ? "No se encontró pw-record (instala pipewire-bin) para capturar audio."
    : !status.model_installed
      ? "Falta el modelo de voz."
      : null;

  return (
    <main>
      <section className="card status" aria-live="polite">
        <span className={`dot ${state.toLowerCase()}`} aria-hidden="true" />
        <strong>{STATE_LABEL[state]}</strong>
        <span className={recording ? "mic-on" : "muted"}>{recording ? "● Micrófono ACTIVO" : "Micrófono inactivo"}</span>
      </section>

      {blocker && (
        <p className="muted small">
          {blocker}{" "}
          {!status.model_installed && status.audio_ok && (
            <button className="link" onClick={onGoSettings}>Ir a Preferencias para descargarlo</button>
          )}
        </p>
      )}

      <button className="cta" onClick={toggle} disabled={busy || !!blocker}>
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
          <button onClick={() => copy(text)} disabled={!text}>Copiar</button>
          <button onClick={insert} disabled={!text} title="Copia el texto; la inserción automática no está disponible">
            Insertar
          </button>
          <button onClick={discard} disabled={!text}>Descartar</button>
        </div>
      </section>
    </main>
  );
}
