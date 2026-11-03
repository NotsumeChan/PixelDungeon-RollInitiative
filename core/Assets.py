import pygame as pg
from pathlib import Path


IMAGES: dict[str, pg.Surface] = {}
ANIMATIONS : dict = {}

def preLoadAssets(path : str) -> None:
    assets = Path(path)
    for entry in assets.iterdir():
        if entry.is_file():
            try:
                img = pg.image.load(entry)
                IMAGES[entry.stem] = img.convert_alpha() if img.get_alpha() else img.convert()
            except pg.error as e:
                print(f"[preload] error cargando {entry}: {e}")
        if entry.is_dir():
            preLoadAssets(entry)

def preLoadAnimations() -> None:
    pass