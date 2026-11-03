"""Registros de contenido leídos de JSON: data/<nombre>.json (clases, mobs, bestiario, ítems, estados, conjuros).
Base de los mods (roadmap/futuro-mods-personajes-red.md): un mod será otra carpeta con los mismos archivos; las
carpetas se leen en orden y lo último pisa a lo anterior. Una entrada con "base": "<id>" copia esa otra entrada y
aplica sus campos encima (personajes o ítems custom basados en los del juego)."""
import json
from pathlib import Path

# ponytail: sólo la carpeta del juego base; los mods agregarán las suyas aquí (instancias, versión 1.x)
DATA_DIRS: list[Path] = [Path(__file__).resolve().parent.parent / "data"]


def load(name: str) -> dict:
    merged: dict = {}
    for folder in DATA_DIRS:
        path = folder / f"{name}.json"
        if path.exists():
            merged.update(json.loads(path.read_text(encoding="utf-8")))
    return {key: _resolve(merged, key, ()) for key in merged}


def _resolve(entries: dict, key: str, seen: tuple):
    entry = entries[key]
    if not (isinstance(entry, dict) and "base" in entry):
        return entry
    if entry["base"] in seen or entry["base"] not in entries:
        raise ValueError(f"{key}: base inválida {entry['base']!r}")
    out = dict(_resolve(entries, entry["base"], seen + (key,)))
    out.update({k: v for k, v in entry.items() if k != "base"})
    return out
