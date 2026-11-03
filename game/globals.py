# Diccionario global del juego: estado compartido entre pantallas y sistemas (seed, piso, nivel actual...).
# Vive en game/ (sin pygame) para que tanto la lógica como las pantallas puedan importarlo.
# Claves en uso: ver docs/game/globals.md
# ponytail: dict plano; si el P2P necesita sincronizarlo o validarlo, envolverlo entonces.
Globals: dict = {}
