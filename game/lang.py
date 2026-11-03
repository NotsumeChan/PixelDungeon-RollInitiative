"""Idioma del contenido. Un texto de contenido (nombre o descripción de ítems, mobs, clases, estados, terreno)
puede ser un str (un solo idioma) o un dict por idioma {"es": ..., "en": ...}; t() elige el del idioma actual.
Un mod trae sus textos en los idiomas que quiera: si falta el actual, cae a inglés y luego al primero que haya.
El idioma se elige en la ventana de configuración y se guarda en Local.config ([general] language)."""

LANGS: dict[str, str] = {"es": "Español", "en": "English"}
FALLBACK = "en"
LANG = "es"


def t(text) -> str:
    if isinstance(text, dict):
        return text.get(LANG) or text.get(FALLBACK) or next(iter(text.values()), "")
    return text


def set_lang(code: str) -> None:
    global LANG
    if code in LANGS:
        LANG = code
