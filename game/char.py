"""actors/Char.java: base de héroe y mobs (vida, combate, movimiento). El combate sigue las reglas de D&D 5e."""
from . import dice
from .actor import Actor
from .buffs import Buff
from .geom import WIDTH


def distance(a: int, b: int) -> int:
    """Level.distance: distancia de Chebyshev (las diagonales cuentan 1)."""
    return max(abs(a % WIDTH - b % WIDTH), abs(a // WIDTH - b // WIDTH))


def adjacent(a: int, b: int) -> bool:
    return distance(a, b) == 1


class Char(Actor):
    in_rounds = True  # en combate actúa por iniciativa (game/actor.py)

    def __init__(self, dungeon, pos: int) -> None:
        super().__init__()
        self.dungeon = dungeon
        self.pos = pos
        self.name = "?"
        self.HT = self.HP = 1
        self.base_speed = 1.0     # velocidad de PD (fuera de combate: tiempo por paso)
        self.base_speed_ft = 30   # velocidad 5e en pies (en combate: casillas de 5 ft por turno)
        # turno de combate 5e (Char.start_turn): movimiento restante, acción disponible, iniciativa
        self.moves, self.has_action, self.init = 0, True, 0
        self.stepped = self.no_moves = False
        self.flying = False
        self.buffs: list[Buff] = []

    # --- stats 5e (se sobreescriben) ---
    def armor_class(self) -> int:
        return 10

    def attack_bonus(self, target: "Char") -> int:
        return 0

    def roll_damage(self, crit: bool, adv: int = 0, target: "Char | None" = None) -> tuple[int, str, tuple | None]:
        """(daño, texto para el log, evento de dado para la animación o None). crit = dados dobles.
        adv = ventaja con la que se atacó (Ataque Furtivo); target = a quién (Marca del cazador)."""
        return 1, "1", None

    def is_unaware(self) -> bool:
        """True si no vio venir el ataque (mob dormido o que no te vio): se le ataca con ventaja."""
        return False

    def advantage(self, target: "Char") -> int:
        """1 = ventaja, -1 = desventaja, 0 = normal."""
        return 1 if target.is_unaware() or target.has_flag("paralysed") or target.is_downed() else 0

    def speed(self) -> float:
        return self.base_speed * self.buff_speed()

    def stealth(self) -> int:
        return 0

    # --- turno de combate 5e: movimiento + 1 acción (las demoras de PD no cuentan en combate) ---
    def speed_ft(self) -> int:
        """Velocidad 5e en pies; los estados la multiplican (Lentitud = la mitad)."""
        return int(self.base_speed_ft * self.buff_speed())

    def initiative_bonus(self) -> int:
        return 0

    def roll_initiative(self) -> int:
        """d20 + bono (DES). Queda en self.init para ordenar a los que se suman después."""
        d = self.dungeon
        nat, _ = dice.d20(d.rng)
        b = self.initiative_bonus()
        self.init = nat + b
        if self in d.heroes:
            d.roll_event(f"Iniciativa {self.name}", self.init, 1 + b, 20 + b)
        return self.init

    def start_turn(self) -> None:
        self.moves = self.speed_ft() // 5
        self.has_action = True
        self.stepped = self.no_moves = False

    def turn_over(self) -> bool:
        return not self.has_action or self.no_moves

    def in_combat(self) -> bool:
        o = self.dungeon.scheduler.order
        return o is not None and self in o

    def can_step(self) -> bool:
        """En combate, sin movimiento restante no camina y su turno termina (la acción en curso sigue el próximo)."""
        if self.in_combat() and self.moves <= 0:
            self.no_moves = True
            return False
        return True

    def spend(self, t: float) -> None:
        """Fuera de combate, tiempo de PD. En combate: lo que sigue a un paso gasta 1 casilla de movimiento; cualquier
        otra cosa gasta la acción y termina el turno (atacar, comer, buscar, esperar, salvación contra la muerte...)."""
        if not self.in_combat():
            super().spend(t)
        elif self.no_moves:
            pass
        elif self.stepped:
            self.moves -= 1
        else:
            self.has_action = False
        self.stepped = False

    def is_alive(self) -> bool:
        return self.HP > 0

    def is_downed(self) -> bool:
        """Inconsciente a 0 PV pero vivo (sólo héroes, ver Hero.damage)."""
        return False

    # --- estados (game/buffs.py) ---
    def buff(self, buff_id: str) -> Buff | None:
        return next((b for b in self.buffs if b.id == buff_id), None)

    def add_buff(self, buff_id: str, duration: float | None = None, level: int = 0, dc: int = 0) -> Buff:
        """Buff.affect + Buff.prolong: si ya lo tiene, se queda con lo mayor (duracion, nivel, CD)."""
        b = self.buff(buff_id)
        if b is None:
            b = Buff(self, buff_id, duration, level, dc)
            self.buffs.append(b)
            self.dungeon.scheduler.add(b)
            if self.dungeon.level.visible[self.pos]:
                self.dungeon.log(f"{self.name}: {b.name.lower()}")
            self.dungeon.status(self, b.name.lower(), "neg")
        else:
            if b.left is not None:
                new = duration if duration is not None else b.data.get("duration")
                b.left = None if new is None else max(b.left, new)
            b.level, b.dc = max(b.level, level), max(b.dc, dc)
        return b

    def has_flag(self, flag: str) -> bool:
        return any(flag in b.data.get("flags", ()) for b in self.buffs)

    def buff_speed(self) -> float:
        s = 1.0
        for b in self.buffs:
            s *= b.data.get("speed", 1.0)
        return s

    # --- tiradas de salvacion (5e) ---
    def save_bonus(self, ability: str) -> int:
        return 0

    def saving_throw(self, ability: str, dc: int, why: str = "") -> tuple[bool, str]:
        """d20 + bono de salvacion contra la CD (sin exito ni fallo automatico con 20 o 1, como en 5e).
        Paralizado: falla sola las de FUE y DES. Devuelve (superada, detalle para el log)."""
        d = self.dungeon
        if self.has_flag("paralysed") and ability in ("STR", "DEX"):
            return False, "paralizado: falla"
        nat, text = dice.d20(d.rng)
        bonus = self.save_bonus(ability)
        ok = nat + bonus >= dc
        if d.level.visible[self.pos]:  # el recuadro muestra el total y la CD ("CON CD11"); borde verde = superada
            # why = de qué es la salvación ("trampa", "paralisis"): va en el recuadro del dado
            label = f"{dice.ABILITY_NAMES[ability]} CD{dc}" + (f" {why}" if why else "")
            d.roll_event(label, nat + bonus, 1 + bonus, 20 + bonus, ok)
        return ok, f"d20 {text}{bonus:+d}={nat + bonus} vs CD {dc}"

    # --- combate 5e ---
    def attack(self, enemy: "Char") -> bool:
        """Tirada de ataque: d20 + bono contra la CA. 20 natural = crítico (dados dobles), 1 natural = falla."""
        d = self.dungeon
        adv = self.advantage(enemy)
        nat, roll_text = dice.d20(d.rng, adv)
        bonus, ac = self.attack_bonus(enemy), enemy.armor_class()
        hit = nat == 20 or (nat != 1 and nat + bonus >= ac)
        line = f"{self.name}: d20 {roll_text}{bonus:+d}={nat + bonus} vs CA {ac}"
        seen = d.sees(self.pos) or d.sees(enemy.pos)
        d.effect("anim", self, "attack", enemy.pos)
        if seen:
            d.roll_event("d20", nat, 1, 20)
        if not hit:
            if seen:  # el resultado se aplica cuando la animación del dado lo muestra
                d.pending.append((self, enemy, False, 0, line + ", falla", f"{self.name} falla"))
            return False
        crit = nat == 20 or (enemy.is_downed() and adjacent(self.pos, enemy.pos))  # 5e: caído y al lado = crítico
        dmg, dmg_text, event = self.roll_damage(crit, adv, enemy)
        dmg = max(dmg, 0)
        if not seen:
            enemy.damage(dmg, self)
            return True
        if event:
            d.roll_event(*event)
        detail = f"{line}, {'CRITICO ' if crit else ''}{dmg_text}={dmg}"
        simple = f"{self.name} golpea a {enemy.name}: {dmg}" + (" (critico)" if crit else "")
        d.pending.append((self, enemy, True, dmg, detail, simple))
        return True

    def damage(self, dmg: int, src) -> None:
        if self.HP <= 0:
            return
        if dmg > 0:
            self.dungeon.status(self, str(dmg), "neg")  # número de daño
        self.HP -= dmg
        if self.HP <= 0:
            self.die(src)

    def die(self, src) -> None:
        self.dungeon.remove_char(self)

    # --- movimiento ---
    def move(self, step: int) -> None:
        """Las puertas se abren al entrar y se cierran al salir (una sola criatura por celda)."""
        level = self.dungeon.level
        old = self.pos
        before = (level.map[old], level.map[step])
        level.leave(old)
        self.pos = step
        level.enter(step)
        if (level.map[old], level.map[step]) != before:
            # se abrió o cerró una puerta: Door.enter/leave -> Dungeon.observe (si no, la vista del héroe queda vieja
            # y lo que pasa del otro lado de la puerta no se ve hasta que el héroe camina)
            self.dungeon.observe()
        self.stepped = True  # el próximo spend() es movimiento (combate 5e)
        if not self.flying:
            self.dungeon.press(step, self)  # trampas
