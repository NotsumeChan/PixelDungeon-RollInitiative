"""Dungeon.java: estado de una partida (seed, pisos, héroes, mobs, planificadores, ítems, mensajes).
Nada global: una instancia por partida (vive en Globals["dungeon"]).
Varios héroes (0.9): cada uno en su piso (hero.depth). level/mobs/scheduler/depth son el piso "en contexto": el que
se está procesando, o el del héroe local (self.hero, el que maneja la UI) fuera de process()."""
from . import ballistica, pathfinder, traps, terrain as T
from .actor import Actor, Scheduler
from .char import adjacent, distance
from .geom import NEIGHBOURS9
from .hero import Hero
from .items import ITEMS, POTION_LOOKS, SCROLL_LOOKS, Generator, Item, HEAP, CHEST, SKELETON, TOMB, make_gold
from .level import SewerLevel
from .mobs import Mob, pick_mob, WANDERING
from .rng import Rng

MAX_DEPTH = 4             # el piso 5 (Goo) queda fuera
TIME_TO_RESPAWN = 50.0    # Level.TIME_TO_RESPAWN
# Dungeon.posNeeded / souNeeded: [piso límite, cantidad acumulada esperada, ...]
STRENGTH_QUOTA = [4, 2, 9, 4, 14, 6, 19, 8, 24, 9]
UPGRADE_QUOTA = [5, 3, 10, 6, 15, 9, 20, 12, 25, 13]
# Reglas de la partida (ventana "Reglas de la partida", estilo configuración de mundo). Se guardan con la partida;
# en red las decidirá el host. Valores por defecto = 5e.
RULES: dict = {
    "dot_death_saves": True,  # el daño por turno (veneno, sangrado, quemadura) a un caído suma fallos de salvación
    "save_half": True,        # superar la salvación de gas tóxico / descarga = la mitad del efecto (False = nada)
}


class Dungeon:
    def __init__(self, seed: int, hero_class: str = "pddnd:warrior", abilities: dict | None = None) -> None:
        self.seed = seed
        self.rng = Rng(seed)  # rng de juego (combate, IA, apariencias); cada nivel genera con su propio rng
        self.depth = 0
        self.messages: list[str] = []
        self.dice_events: list[tuple] = []  # (etiqueta, valor, mínimo, máximo) que el héroe ve tirar; la UI los consume
        # ataques vistos esperando a que la animación muestre el dado: (atacante, objetivo, golpe, daño, detalle, simple),
        # o una función sin argumentos (salvaciones de trampas) que se llama al resolver
        self.pending: list[tuple] = []
        # efectos para el render, que los consume: ("status", char, texto, tipo) = texto flotante (CharSprite.showStatus),
        # ("checked", celda) = búsqueda (CheckedCell), ("leaves", celda) = pasto pisado
        self.fx: list[tuple] = []
        self.rules: dict = dict(RULES)  # reglas de la partida (ver RULES)
        self.verbose_rolls = False  # opción de desarrollador: el log muestra el cálculo completo de cada tirada
        # identificación: qué pociones/pergaminos se conocen y con qué apariencia se ve cada uno
        self.known: set[str] = set()
        self.looks: dict[str, tuple[str, int]] = {}
        for category, looks in (("potion", POTION_LOOKS), ("scroll", SCROLL_LOOKS)):
            looks = list(looks)
            self.rng.shuffle(looks)
            ids = [i for i, d in ITEMS.items() if d["category"] == category]
            # ponytail: si un mod agrega más de 12 pociones/pergaminos faltan apariencias; habrá que ampliar la lista
            self.looks.update(zip(ids, looks))
        self.generator = Generator()
        self.potions_of_strength = 0
        self.scrolls_of_upgrade = 0
        self.mobs: list[Mob] = []
        self.floors: dict[int, tuple] = {}  # pisos visitados: depth -> (level, mobs, scheduler)
        self.heroes: list[Hero] = []
        self.hero: Hero = None  # el héroe local (el primero, salvo set_local)
        self.add_hero(hero_class, abilities)

    def add_hero(self, hero_class: str, abilities: dict | None = None) -> Hero:
        """Suma un héroe a la partida, en la entrada del piso 1."""
        hero = Hero(self, hero_class, -1, abilities)
        self.heroes.append(hero)
        self.hero = self.hero or hero
        self.go_to(hero, 1)
        return hero

    def remove_hero(self, hero: Hero) -> None:
        """Un jugador se va de la partida (red): sale de su piso con lo suyo (estados, regeneración, hambre).
        ponytail: su personaje no vuelve a su PC todavía (paso 5)."""
        if hero is self.hero or hero not in self.heroes:
            return
        self.enter(hero.depth)
        for a in (hero, *hero.ticks, *hero.buffs):
            self.scheduler.remove(a)
        self.heroes.remove(hero)
        self.observe()
        self.enter(self.hero.depth)

    def here(self) -> list[Hero]:
        """Héroes del piso en contexto (vivos o no)."""
        return [h for h in self.heroes if h.depth == self.depth]

    def set_local(self, hero: Hero) -> None:
        """Cambia el héroe que maneja la UI (modo de prueba de 2 héroes) y pone su piso en contexto."""
        self.hero = hero
        self.enter(hero.depth)

    # --- pisos ---
    def enter(self, depth: int) -> None:
        """Pone en contexto un piso ya creado."""
        self.depth = depth
        self.level, self.mobs, self.scheduler = self.floors[depth]

    def descend(self, hero: Hero) -> None:
        if hero.depth >= MAX_DEPTH:
            self.log("Fin de la demo: el piso 5 no está incluido")
            return
        self.go_to(hero, hero.depth + 1)

    def ascend(self, hero: Hero) -> None:
        if hero.depth <= 1:
            self.log("No puedes salir de la mazmorra")
            return
        self.go_to(hero, hero.depth - 1)

    def go_to(self, hero: Hero, depth: int) -> None:
        """Cambia de piso a un héroe (InterlevelScene DESCEND/ASCEND). Los pisos quedan en self.floors: al volver
        siguen sus mobs, ítems y lo explorado. Un piso sin héroes vivos no se procesa (su tiempo no avanza).
        El héroe se lleva lo suyo: estados, regeneración y hambre (hero.ticks)."""
        going_down, old = depth > hero.depth, hero.depth
        hero.depth = depth
        if old:
            self.enter(old)
            for a in (hero, *hero.ticks, *hero.buffs):
                self.scheduler.remove(a)
            self.observe()  # el piso que deja ya no ve lo que veía este héroe
        hero.pos = -1  # que free_near no lo cuente como ocupando la escalera
        hero.action = None
        hero.visible_enemies = []
        self.pending = []
        if depth in self.floors:
            self.enter(depth)
            hero.pos = self.free_near(self.level.entrance if going_down else self.level.exit)
            self.scheduler.add(hero)
        else:
            self.generator.reset()
            self.floors[depth] = (SewerLevel(self.seed, depth), [], Scheduler())
            self.enter(depth)
            hero.pos = self.level.entrance
            hero.recover()  # ponytail: hasta que haya descanso corto/largo, se recupera en cada piso nuevo
            self.scheduler.add(hero)  # primero: en empates actúa antes que nadie
            self.create_mobs()
            self.create_items()
            self.scheduler.add(Respawner(self))
        for a in (*hero.ticks, *hero.buffs):
            self.scheduler.add(a)
        self.observe()
        self.log(f"Piso {depth}" if len(self.heroes) == 1 else f"{hero.name}: piso {depth}")

    def free_near(self, cell: int) -> int:
        """cell, o una vecina libre y transitable (otro héroe o un mob ya está en la escalera)."""
        for c in (cell + n for n in NEIGHBOURS9):
            if self.level.passable[c] and self.find_char(c) is None:
                return c
        return cell  # ponytail: todo ocupado, comparte celda hasta que alguien se mueva

    def observe(self) -> None:
        """Dungeon.observe: la vista de cada héroe del piso (hero.fov); level.visible es la unión (lo que se dibuja)."""
        hs = self.here()
        for h, f in zip(hs, self.level.observe(*(h.pos for h in hs))):
            h.fov = f

    def n_mobs(self, rng) -> int:
        return 2 + self.depth % 5 + rng.Int(3)

    def create_mobs(self) -> None:
        """RegularLevel.createMobs, con el rng del nivel (mismo nivel = mismos mobs)."""
        rng = self.level.rng
        for _ in range(self.n_mobs(rng)):
            key = pick_mob(rng, self.depth)
            pos = -1
            while pos == -1:
                pos = self.level.random_respawn_cell(rng, self.find_char)
            self.add_mob(Mob(self, key, pos))

    def _quota(self, rng, quota: list[int], number: int) -> bool:
        """Dungeon.chance: garantiza ~N ítems clave antes de cierto piso."""
        for i in range(0, len(quota), 2):
            if self.depth <= quota[i]:
                return rng.Float() < (quota[i + 1] - number) / (quota[i] - self.depth + 1)
        return False

    def create_items(self) -> None:
        """Level.create (ítems obligatorios) + RegularLevel.createItems + tumbas del cementerio."""
        level, rng, gen = self.level, self.level.rng, self.generator
        to_spawn = [gen.random_of("food", rng, self)]
        if self._quota(rng, STRENGTH_QUOTA, self.potions_of_strength):
            to_spawn.append(Item("pddnd:potion_strength"))
            self.potions_of_strength += 1
        if self._quota(rng, UPGRADE_QUOTA, self.scrolls_of_upgrade):
            to_spawn.append(Item("pddnd:scroll_upgrade"))
            self.scrolls_of_upgrade += 1

        n = 3
        while rng.Float() < 0.4:
            n += 1
        for _ in range(n):
            t = rng.Int(20)
            # ponytail: el caso 5 (mimic en pisos > 1) sale como cofre normal hasta que exista el Mimic
            type_ = SKELETON if t == 0 else CHEST if t <= 5 else HEAP
            level.drop(gen.random(rng, self), level.random_drop_cell(rng), type_)
        for item in to_spawn:
            level.drop(item, level.random_drop_cell(rng), HEAP)
        for pos, has_item in level.tombs:
            level.drop(gen.random(rng, self) if has_item else make_gold(rng, self.depth), pos, TOMB)

    def add_mob(self, mob: Mob, delay: float = 0.0) -> None:
        self.mobs.append(mob)
        self.scheduler.add(mob, delay)

    def press(self, cell: int, ch) -> None:
        """Level.press / mobPress: pasto alto y trampas."""
        level = self.level
        if level.map[cell] == T.HIGH_GRASS:  # HighGrass.trample (sin semillas ni rocío)
            level.set(cell, T.GRASS)
            if level.visible[cell]:
                self.effect("leaves", cell)
            self.observe()  # el pasto alto tapaba la vista
        traps.press(self, cell, ch)

    # --- personajes ---
    def chars(self) -> list:
        return [*self.here(), *self.mobs]

    def find_char(self, cell: int):
        for c in self.chars():
            if c.pos == cell and c.is_alive():
                return c
        return None

    def remove_char(self, ch) -> None:
        if ch in self.mobs:
            self.mobs.remove(ch)
        self.scheduler.remove(ch)

    def choose_enemy(self, mob):
        """El héroe vivo más cercano que el mob ve; si no ve a ninguno, el más cercano (para seguir su rastro).
        En empate, el primero de self.heroes (determinista)."""
        hs = [h for h in self.here() if h.is_alive()]
        seen = [h for h in hs if mob.fov[h.pos]]
        # prefiere a los que están en pie; a un caído lo remata sólo si no hay otro
        return min(seen or hs, key=lambda h: (h.is_downed(), distance(mob.pos, h.pos)), default=None)

    def shot(self, frm: int, to: int) -> tuple[int, int]:
        """Línea de tiro de un héroe (proyectil o conjuro): pasa por los otros héroes sin golpearlos (no hay fuego
        amigo) y se frena en el primer enemigo o pared. Devuelve (celda donde se frena, cobertura): 5e, una criatura
        en medio da media cobertura = +2 a la CA del objetivo (aquí, -2 a la tirada)."""
        allies = []

        def block(cell: int):
            ch = self.find_char(cell)
            if ch in self.heroes:
                allies.append(ch)
                return None
            return ch

        stop = ballistica.cast(self.level, frm, to, block)
        return stop, (2 if allies else 0)

    def find_path(self, ch, frm: int, to: int, passable: list[bool], visible: list[bool]) -> int:
        """Dungeon.findPath: un paso hacia `to` evitando a las criaturas que `ch` ve."""
        level = self.level
        if adjacent(frm, to):
            return to if self.find_char(to) is None and (passable[to] or level.avoid[to]) else -1
        p = [a or b for a, b in zip(passable, level.avoid)] if ch.flying else list(passable)
        for c in self.chars():
            if visible[c.pos]:
                p[c.pos] = False
        return pathfinder.get_step(frm, to, p)

    # --- turnos y mensajes ---
    def process(self) -> None:
        """Corre los turnos de cada piso con héroes vivos. Se frena mientras haya un ataque visto sin resolver (lo
        resuelve resolve_pending). Al final queda en contexto el piso del héroe local."""
        for depth in dict.fromkeys(h.depth for h in self.heroes if h.is_alive()):  # sin repetir, en orden
            self.enter(depth)
            s = self.scheduler  # go_to() corre dentro de process(): al cambiar de piso se corta el loop del viejo
            s.process(stop=lambda: bool(self.pending) or self.scheduler is not s)
        self.enter(self.hero.depth)

    def blocking(self):
        """El héroe del piso en contexto al que le toca actuar (el planificador espera su acción), o None."""
        a = self.scheduler.current()  # en combate, el de la iniciativa
        return a if a in self.heroes else None

    def defer(self, p) -> None:
        """Agrega p a pending (espera al dado). Fuera del piso local no hay dado que mirar: se aplica ya, con el
        contexto correcto (mientras se procesa un piso, pending está vacío: process() frena si no lo está)."""
        self.pending.append(p)
        if self.depth != self.hero.depth:
            self.resolve_pending()

    def resolve_pending(self) -> None:
        """Aplica los ataques que la UI ya mostró. Sin rng nuevo: el orden de tiradas no cambia (determinista)."""
        pending, self.pending = self.pending, []
        for p in pending:
            if callable(p):
                p()
                continue
            attacker, enemy, hit, dmg, detail, simple = p
            self.log(detail if self.verbose_rolls else simple)
            if hit:
                enemy.damage(dmg, attacker)
            else:
                self.status(enemy, "esquiva", "neutral")

    def roll_event(self, label: str, value: int, lo: int, hi: int, ok: bool | None = None) -> None:
        """ok: resultado de una salvación (borde verde / rojo); None en ataques y daño."""
        if self.depth != self.hero.depth:
            return
        self.dice_events.append((label, value, lo, hi, ok))
        del self.dice_events[:-20]

    def status(self, ch, text: str, kind: str = "default") -> None:
        """CharSprite.showStatus: texto flotante sobre ch si se ve. kind: default, pos, neg, warn, neutral."""
        if self.level.visible[ch.pos]:
            self.effect("status", ch, text, kind)

    def sees(self, cell: int) -> bool:
        """Lo que pasa en cell se muestra en pantalla (piso del héroe local y a la vista). Lo que se muestra espera a la
        animación del dado (pending) y se resuelve en el piso local: lo de otros pisos se aplica en el acto."""
        return self.depth == self.hero.depth and self.level.visible[cell]

    def effect(self, *fx) -> None:
        if self.depth != self.hero.depth:  # sólo se dibuja el piso del héroe local
            return
        self.fx.append(fx)
        del self.fx[:-40]

    def log(self, msg: str) -> None:
        self.messages.append(msg)
        del self.messages[:-50]


class Respawner(Actor):
    """Level.respawner: cada 50 de tiempo, si faltan mobs, aparece uno nuevo (despierto) fuera de la vista."""

    def __init__(self, dungeon: Dungeon) -> None:
        super().__init__()
        self.dungeon = dungeon

    def act(self) -> None:
        d = self.dungeon
        if len(d.mobs) < d.n_mobs(d.rng):
            pos = d.level.random_respawn_cell(d.rng, d.find_char)
            if pos != -1:  # sólo se procesan pisos con héroes vivos
                d.add_mob(Mob(d, pick_mob(d.rng, d.depth), pos, WANDERING))
        self.spend(TIME_TO_RESPAWN)
