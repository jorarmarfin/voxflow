import { useEffect, useState } from "react";
import { api, errMsg, type Device, type Diagnostics, type ModelStatus, type OutputMode, type Progress, type Settings } from "../lib/api";

interface Props {
  settings: Settings;
  onSaved: (s: Settings) => void;
  onNotice: (m: string | null) => void;
  onModelChange: () => void;
}

const mb = (n: number) => `${(n / 1048576).toFixed(0)} MB`;

export default function SettingsView({ settings, onSaved, onNotice, onModelChange }: Props) {
  const [draft, setDraft] = useState<Settings>(settings);
  const [devices, setDevices] = useState<Device[]>([]);
  const [models, setModels] = useState<ModelStatus[]>([]);
  const [progress, setProgress] = useState<Progress | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [diag, setDiag] = useState<Diagnostics | null>(null);
  const set = <K extends keyof Settings>(k: K, v: Settings[K]) => setDraft({ ...draft, [k]: v });
  const model = models.find((m) => m.id === draft.model);

  const refreshModels = () => api.modelStatus().then(setModels).catch((e) => onNotice(errMsg(e)));

  useEffect(() => {
    api.listDevices().then(setDevices).catch((e) => onNotice(errMsg(e)));
    void refreshModels();
    api.getDiagnostics().then(setDiag).catch(() => setDiag(null));
    const subs = [
      api.onProgress(setProgress),
      api.onDownloadDone((d) => {
        setDownloading(false);
        setProgress(null);
        onNotice(d.ok ? "Modelo instalado y verificado." : (d.error ?? "Descarga fallida."));
        void refreshModels();
        onModelChange();
      }),
    ];
    return () => {
      subs.forEach((p) => p.then((un) => un()));
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const save = async () => {
    try {
      onSaved(await api.saveSettings(draft));
      onNotice("Preferencias guardadas.");
      onModelChange();
    } catch (e) {
      onNotice(errMsg(e));
    }
  };
  const download = () => {
    setDownloading(true);
    api.downloadModel(draft.model).catch((e) => {
      setDownloading(false);
      onNotice(errMsg(e));
    });
  };
  const remove = async () => {
    try {
      await api.deleteModel(draft.model);
      await refreshModels();
      onModelChange();
    } catch (e) {
      onNotice(errMsg(e));
    }
  };
  const pct = progress && progress.total ? Math.round((progress.downloaded / progress.total) * 100) : null;

  return (
    <main>
      <section className="card form">
        <label htmlFor="device">Micrófono</label>
        <select id="device" value={draft.device_id ?? ""} onChange={(e) => set("device_id", e.target.value || null)}>
          <option value="">Predeterminado del sistema</option>
          {devices.map((d) => (
            <option key={d.id} value={d.id}>{d.label}</option>
          ))}
        </select>

        <label htmlFor="model">Modelo</label>
        <select id="model" value={draft.model} onChange={(e) => set("model", e.target.value)}>
          <option value="base">base (~142 MB)</option>
        </select>
        <div className="actions">
          {model?.installed ? (
            <>
              <span className="muted small">Instalado ({mb(model.size_bytes)}), verificado</span>
              <button onClick={remove}>Eliminar</button>
            </>
          ) : downloading ? (
            <>
              <progress value={pct ?? undefined} max={100} aria-label="Progreso de descarga" />
              <span className="small">{pct !== null ? `${pct}%` : "…"}</span>
              <button onClick={() => api.cancelDownload()}>Cancelar</button>
            </>
          ) : (
            <button onClick={download}>Descargar modelo</button>
          )}
        </div>

        <label htmlFor="lang">Idioma</label>
        <select id="lang" value={draft.language} onChange={(e) => set("language", e.target.value)}>
          <option value="es">Español</option>
          <option value="en">English</option>
        </select>

        <label htmlFor="out">Modo de salida</label>
        <select id="out" value={draft.output_mode} onChange={(e) => set("output_mode", e.target.value as OutputMode)}>
          <option value="preview">Vista previa antes de copiar</option>
          <option value="clipboard">Copiar directo al portapapeles</option>
          <option value="insert" disabled={!diag?.auto_insert}>
            Insertar en la ventana activa (con el atajo)
          </option>
        </select>

        {diag?.session === "windows" && (
          <>
            <label htmlFor="hotkey">Atajo global</label>
            <input
              id="hotkey"
              type="text"
              value={draft.hotkey}
              onChange={(e) => set("hotkey", e.target.value)}
              placeholder="Ctrl+Alt+V"
              maxLength={40}
            />
          </>
        )}

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
          La única conexión de red es la descarga explícita del modelo.
        </p>
        <h2>Atajo de teclado</h2>
        {diag?.session === "windows" ? (
          <p className="muted small">
            Pulsa el atajo global ({draft.hotkey}) desde cualquier aplicación: la primera vez empieza a grabar y la
            siguiente detiene y transcribe. También funciona <code>voxflow.exe --toggle</code>.
          </p>
        ) : (
          <p className="muted small">
            Asigna en tu escritorio un atajo que ejecute <code>voxflow --toggle</code>: la primera vez empieza a
            grabar y la siguiente detiene y transcribe. En GNOME: <code>scripts/install-gnome-shortcut.sh</code>
            (ver README).
          </p>
        )}
        <h2>Diagnóstico</h2>
        {diag && (
          <ul className="muted small">
            <li>
              Sistema:{" "}
              {{ wayland: "Linux (Wayland)", x11: "Linux (X11)", windows: "Windows", unknown: "desconocido" }[diag.session]}
            </li>
            <li>Portapapeles: {diag.clipboard_tool ?? "no disponible (Linux: instala wl-clipboard o xclip)"}</li>
            <li>Notificaciones: {diag.notifications ? "notify-send" : "no disponible (Linux: instala libnotify-bin)"}</li>
            <li>
              Inserción en el cursor:{" "}
              {diag.auto_insert
                ? `disponible (${diag.insert_tool}); solo con el atajo, pega en la ventana que tenías activa al empezar`
                : "no disponible (Linux: requiere X11, xdotool y xprop); el texto se copia y se pega con Ctrl+V"}
            </li>
            <li>Registro de diagnóstico: {diag.log_path ?? "no disponible"} (nunca contiene audio ni texto dictado)</li>
          </ul>
        )}
      </section>
    </main>
  );
}
