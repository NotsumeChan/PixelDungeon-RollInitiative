"""Lista de personajes creados, guardada en disco (JSON) en la carpeta de datos del usuario.
Personaje = {"id", "name", "class", "abilities", "created"}. Las partidas en curso todavía no se guardan."""
import json
import os
import sys
import uuid
from datetime import datetime
from pathlib import Path

APP_NAME = "PixelDND"
FILE_NAME = "characters.json"


def data_dir() -> Path:
    """Windows: %APPDATA%/PixelDND. Linux: $XDG_DATA_HOME/PixelDND (o ~/.local/share/PixelDND).
    ponytail: Android tendrá su propia ruta cuando se porte."""
    if sys.platform.startswith("win"):
        base = Path(os.environ.get("APPDATA") or Path.home())
    else:
        base = Path(os.environ.get("XDG_DATA_HOME") or Path.home() / ".local" / "share")
    path = base / APP_NAME
    path.mkdir(parents=True, exist_ok=True)
    return path


def load() -> list[dict]:
    path = data_dir() / FILE_NAME
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
        return data if isinstance(data, list) else []
    except FileNotFoundError:
        return []
    except (json.JSONDecodeError, UnicodeDecodeError):
        # archivo roto: se aparta (no se pisa) para no perder los personajes
        path.replace(path.with_name(f"{FILE_NAME}.roto-{datetime.now():%Y%m%d%H%M%S}"))
        return []


def save(chars: list[dict]) -> None:
    """Escritura atómica: si el juego se corta a mitad, el archivo anterior queda intacto."""
    path = data_dir() / FILE_NAME
    tmp = path.with_suffix(".tmp")
    tmp.write_text(json.dumps(chars, ensure_ascii=False, indent=1), encoding="utf-8")
    tmp.replace(path)


def new_character(name: str, class_id: str, abilities: dict) -> dict:
    return {"id": uuid.uuid4().hex, "name": name, "class": class_id, "abilities": dict(abilities),
            "created": datetime.now().isoformat(timespec="seconds")}
