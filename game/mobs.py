"""Mobs de las Alcantarillas: actors/mobs/*.java + Bestiary.java.
Los mobs son DATOS (MOBS, IDs "namespace:nombre") + una clase Mob genérica: un addon podrá agregar entradas."""
from . import fov, registry
from .actor import TICK
from .char import Char, distance, adjacent
from .geom import WIDTH
from .items import Item, GOLD, make_gold
from .lang import t

# sprite: archivo en Assets/Characters, frame = (ancho, alto) de cada frame (sprites/*Sprite.java)
# anims: animaciones del sprite (las usa el render): nombre -> [fps, [frames], se repite] (*Sprite.java).
# loot: (id de ítem, probabilidad) al morir. desc: texto de la ventana de examinar (Mob.description)


MOBS: dict[str, dict] = registry.load("mobs")  # data/mobs.json
# ponytail: sin habilidades especiales (Swarm se divide, Skeleton explota y suelta armas, Thief roba)

# Bestiary.java: piso -> [(mob, peso)]
BESTIARY: dict[int, list] = {int(k): v for k, v in registry.load("bestiary").items()}  # data/bestiary.json


def pick_mob(rng, depth: int) -> str:
    table = BESTIARY[depth]
    return table[rng.chances([w for _, w in table])][0]


def mob_ac(s: dict) -> int:
    """Conversión PD -> 5e: la evasión y la reducción de daño del original se vuelven CA.
    Rat 10, Gnoll 11, Crab 12, Swarm 10, Skeleton 15, Thief 15 (base 8: con base 10 los primeros pisos eran
    muy duros). Ajustar aquí si el balance no convence."""
    return 8 + (s["defense"] + s["dr"]) // 2


def mob_attack_bonus(s: dict) -> int:
    """Conversión PD -> 5e: precisión del original / 2. Rat +4, Gnoll +5, el resto +6."""
    return s["attack"] // 2


SLEEPING, WANDERING, HUNTING = "SLEEPING", "WANDERING", "HUNTING"
MOB_STATES = {SLEEPING: {"es": "Está durmiendo", "en": "Sleeping"},  # AiState.status
              WANDERING: {"es": "Está deambulando", "en": "Wandering"},
              HUNTING: {"es": "Te está cazando", "en": "Hunting you"}}
TIME_TO_WAKE_UP = 1.0


class Mob(Char):
    def __init__(self, dungeon, key: str, pos: int, state: str = SLEEPING) -> None:
        super().__init__(dungeon, pos)
        s = self.stats = MOBS[key]
        self.key = key
        self.HT = self.HP = s["HT"]
        self.base_speed = s.get("speed", 1.0)
        self.base_speed_ft = s.get("speed_ft", 30)
        self.flying = s.get("flying", False)
        self.state = state
        self.target = -1
        self.enemy = None
        self.enemy_seen = False
        self.alerted = False
        self.fov: list[bool] = []

    @property
    def name(self) -> str:
        return t(self.stats["name"])  # siempre en el idioma actual

    @name.setter
    def name(self, value) -> None:
        pass  # Char.__init__ pone "?"; el nombre de un mob sale de MOBS

    # --- combate 5e: las stats de PD (attack/defense/dr) se convierten (ver mob_ac / mob_attack_bonus) ---
    def armor_class(self) -> int:
        return mob_ac(self.stats)

    def attack_bonus(self, target) -> int:
        return mob_attack_bonus(self.stats)

    def save_bonus(self, ability: str) -> int:
        """Los mobs no tienen características: "saves" en MOBS ({"CON": 3}) o precisión del original / 4."""
        return self.stats.get("saves", {}).get(ability, self.stats["attack"] // 4)

    def beckon(self, cell: int) -> None:
        """Mob.beckon (trampa de alarma): lo despierta y lo manda a la celda."""
        if self.state != HUNTING:
            self.state = WANDERING
        self.target = cell

    def fighting(self) -> bool:
        return self.state == HUNTING and self.enemy_seen and self.is_alive()

    def initiative_bonus(self) -> int:
        return self.stats.get("initiative", 0)  # mod de DES 5e del mob (data/mobs.json)

    def is_unaware(self) -> bool:
        return not self.enemy_seen  # dormido o sin haberte visto: le atacas con ventaja

    def roll_damage(self, crit: bool, adv: int = 0, target=None) -> tuple[int, str, tuple]:
        """El daño conserva el rango del original (NormalIntRange); con crítico se tira dos veces."""
        rng, (lo, hi) = self.dungeon.rng, self.stats["damage"]
        k = 2 if crit else 1
        rolls = [rng.NormalIntRange(lo, hi) for _ in range(k)]
        return sum(rolls), f"{lo}-{hi}({'+'.join(map(str, rolls))})", (f"{lo}-{hi}", sum(rolls), lo * k, hi * k)

    def exp_for(self, hero) -> int:
        return self.stats["exp"] if hero.lvl <= self.stats["max_lvl"] else 0

    # --- turno (Mob.act + AiState) ---
    def act(self) -> None:
        self.fov = fov.cast_shadow(self.pos % WIDTH, self.pos // WIDTH, self.dungeon.level.los_blocking)
        just_alerted, self.alerted = self.alerted, False
        if self.has_flag("paralysed"):
            self.enemy_seen = False
            self.spend(TICK)
            return
        e = self.enemy = self.dungeon.choose_enemy(self)
        in_fov = e is not None and e.is_alive() and self.fov[e.pos]
        if self.state == SLEEPING:
            self._sleeping(in_fov)
        elif self.state == WANDERING:
            self._wandering(in_fov, just_alerted)
        else:
            self._hunting(in_fov)

    def _sleeping(self, in_fov: bool) -> None:
        e, rng = self.enemy, self.dungeon.rng
        if in_fov and rng.Int(distance(self.pos, e.pos) + e.stealth() + (2 if e.flying else 0)) == 0:
            self.enemy_seen = True
            self.state = HUNTING
            self.target = e.pos
            self.spend(TIME_TO_WAKE_UP)
        else:
            self.enemy_seen = False
            self.spend(TICK)

    def _wandering(self, in_fov: bool, just_alerted: bool) -> None:
        e, rng = self.enemy, self.dungeon.rng
        if in_fov and (just_alerted or rng.Int(distance(self.pos, e.pos) // 2 + e.stealth()) == 0):
            # sin gastar tiempo: el planificador lo vuelve a elegir y actúa ya en HUNTING (igual que Java)
            self.enemy_seen = True
            self.state = HUNTING
            self.target = e.pos
        else:
            self.enemy_seen = False
            if self.target != -1 and self.get_closer(self.target):
                self.spend(1 / self.speed())
            else:
                self.target = self.dungeon.level.random_destination(rng)
                self.spend(TICK)

    def _hunting(self, in_fov: bool) -> None:
        e = self.enemy
        self.enemy_seen = in_fov
        if in_fov and adjacent(self.pos, e.pos):
            self.attack(e)
            self.spend(1.0)  # attackDelay
            return
        if in_fov:
            self.target = e.pos
        if self.target != -1 and self.get_closer(self.target):
            self.spend(1 / self.speed())
        else:
            self.spend(TICK)
            self.state = WANDERING
            self.target = self.dungeon.level.random_destination(self.dungeon.rng)

    def get_closer(self, target: int) -> bool:
        if not self.can_step():
            return True  # sin movimiento en este turno de combate: el spend() siguiente no gasta nada
        step = self.dungeon.find_path(self, self.pos, target, self.dungeon.level.passable, self.fov)
        if step == -1:
            return False
        self.move(step)
        return True

    # --- daño y muerte ---
    def damage(self, dmg: int, src) -> None:
        if self.state == SLEEPING:
            self.state = WANDERING
        self.alerted = True
        super().damage(dmg, src)

    def die(self, src) -> None:
        super().die(src)
        self.dungeon.effect("die", self)  # el render deja el cadáver con su animación de muerte
        d = self.dungeon
        if d.level.visible[self.pos]:
            d.log(f"{self.name} muere")
        # XP repartida entre los héroes vivos del piso (redondeo hacia arriba) y +1 al que dio el último golpe.
        # Con un solo héroe queda igual que el original.
        alive = [h for h in d.here() if h.is_alive()]
        for h in alive:
            share = -(-self.exp_for(h) // len(alive))
            if share and h is src and len(alive) > 1:
                share += 1
            h.earn_exp(share)
        # Mob.dropLoot: sólo si los héroes no los superan por mucho (cuenta el de menor nivel)
        loot = self.stats.get("loot")
        low = min((h.lvl for h in d.here()), default=0)
        if loot and low <= self.stats["max_lvl"] + 2 and d.rng.Float() < loot[1]:
            d.level.drop(make_gold(d.rng, d.depth) if loot[0] == GOLD else Item(loot[0]), self.pos)
