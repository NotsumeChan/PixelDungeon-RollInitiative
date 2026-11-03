import sys
import pygame as pg
from time import time

def main() -> None:
    pg.mixer.init()
    pg.init()

    #from core import preload
    #preload(scale=1)

    from screens import HomeScreen 
    Delta = time()
    HomeScreen(Delta)

if __name__ == "__main__":
    main()
    pg.quit()
    pg.mixer.quit()
    sys.exit()