#!/usr/bin/env bash
# Crea (o quita) un atajo personalizado de GNOME que ejecuta `voxflow --toggle`.
# Solo añade/quita SU entrada; conserva tus demás atajos personalizados.
# Uso: scripts/install-gnome-shortcut.sh [--binding '<Super><Alt>v'] [--command voxflow]
#      scripts/install-gnome-shortcut.sh --remove
# Con AppImage: --command '/ruta/VoxFlow_0.1.0_amd64.AppImage'   (el comando se ejecuta con "--toggle")
set -euo pipefail
binding='<Super><Alt>v'; command_path='voxflow'; remove=0
while [ $# -gt 0 ]; do
  case "$1" in
    --binding) binding="$2"; shift 2 ;;
    --command) command_path="$2"; shift 2 ;;
    --remove) remove=1; shift ;;
    *) echo "argumento desconocido: $1" >&2; exit 2 ;;
  esac
done
command -v gsettings >/dev/null || { echo "gsettings no disponible (¿no es GNOME?)" >&2; exit 1; }
schema=org.gnome.settings-daemon.plugins.media-keys
base=/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings
mine="$base/voxflow/"
current="$(gsettings get $schema custom-keybindings)"
list="$(python3 - "$current" "$mine" "$remove" <<'PY'
import ast, sys
cur, mine, remove = sys.argv[1], sys.argv[2], sys.argv[3] == "1"
cur = cur.replace("@as ", "")
items = [x for x in ast.literal_eval(cur) if x != mine]
if not remove:
    items.append(mine)
print(repr(items))
PY
)"
gsettings set $schema custom-keybindings "$list"
key="org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$mine"
if [ "$remove" = 1 ]; then
  gsettings reset "$key" name; gsettings reset "$key" command; gsettings reset "$key" binding
  echo "Atajo de VoxFlow eliminado."
else
  gsettings set "$key" name 'VoxFlow'
  gsettings set "$key" command "$command_path --toggle"
  gsettings set "$key" binding "$binding"
  echo "Atajo creado: $binding → $command_path --toggle"
fi
