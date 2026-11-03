# screens/charSprite.py

Animaciones de personajes: `sprites/CharSprite.java`, `HeroSprite.java`, `MobSprite.java` y el `*Sprite.java` de cada mob.

## Datos
- Animación = `(fps, [frames], se repite)`. Héroe: `HERO_ANIMS` (igual en las 4 clases): quieto, correr, morir, atacar,
  operar (abrir, buscar, comer, beber) y leer. Mobs: `MOBS[id]["anims"]` en `game/mobs.py` (quieto, correr, atacar,
  morir), **datos para mods**. Un mob sin `anims` queda quieto en su primer frame.
- `Frame(ch, i, flip)`: héroe = fila según el tier de la armadura (`HeroSprite.tiers`, sin armadura = fila 0); mob = los
  frames corren de izquierda a derecha y bajan de fila (`TextureFilm`). Cacheado, con la versión espejada.

## Estado (`CharAnim`) y reglas (`Update`)
- Por personaje: animación actual, tiempo y `flip` (mira a la izquierda, `CharSprite.turnTo`).
- Muerto -> `die` (el héroe se queda en el último frame). Una de `ONESHOT` (atacar, operar, leer) se termina antes de
  cambiar. Si su sprite todavía viaja hacia la celda -> `run`, girado hacia donde va. Si no -> `idle`.
- `Act(anims, ch, tipo, celda)`: lo dispara `d.fx` (`("anim", ch, tipo, celda)`); se gira hacia la celda.
- Cadáveres: `Mob.die` emite `("die", mob)`. El mob ya salió de la partida; la pantalla guarda su sprite donde estaba,
  reproduce `die` y lo desvanece en `FADE_TIME` = 3 s (`MobSprite.onComplete`). `CorpseAlpha` da la transparencia.
- Todo avanza con `dt`.

## Parálisis
Un personaje con el flag `paralysed` congela su animación (`CharSprite.paralyse`) hasta que se libera.

- Un héroe caído (`is_downed`) reproduce `die` y queda tendido en el último frame; al levantarse vuelve a `idle`.
