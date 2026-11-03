"""actors/hero/Hero.java + HeroClass.java + Belongings.java + buffs/Regeneration.java + buffs/Hunger.java,
con las reglas de combate de D&D 5e (dados, CA, competencias, características).
Las clases son DATOS (HERO_CLASSES, IDs "namespace:nombre"): un addon podrá agregar o reemplazar entradas."""
from . import dice, spells, ballistica, registry
from .actor import Actor
from .buffs import Buff
from .char import Char, adjacent, distance
from .geom import NEIGHBOURS9, LENGTH
from .lang import t
from .items import Item, EFFECTS, TARGETS, HEAP, CHEST, TOMB, SKELETON, weapon_dice, armor_ac, item_name, item_image

HERO_CLASSES: dict[str, dict] = registry.load("classes")  # data/classes.json
# ponytail: faltan Oleada de Acción, estilos de combate, subclases, el arco del Explorador y los ítems propios de
# cada clase (varita del Mago, bumerán de la Cazadora); los conjuros no piden concentración ni foco

BACKPACK_SIZE = 19
TIME_TO_REST = 1.0
TIME_TO_SEARCH = 2.0
TIME_TO_PICK_UP = 1.0
TIME_TO_OPEN = 1.0     # Key.TIME_TO_UNLOCK
TIME_TO_THROW = 1.0    # Item.TIME_TO_THROW (por la velocidad del arma)
OPEN_MSG = {CHEST: {"es": "Abres el cofre", "en": "You open the chest"},
            TOMB: {"es": "Abres la tumba", "en": "You open the tomb"},
            SKELETON: {"es": "Revisas los restos", "en": "You search the remains"}}
TIME_TO_DROP = 0.5
TIME_TO_EAT = 3.0
TIME_TO_USE = 1.0      # beber / leer
TIME_TO_EQUIP = 1.0    # arma; armadura = el doble
REGENERATION_DELAY = 10.0
HUNGER_STEP = 10.0
HUNGRY = 260.0
STARVING = 360.0


class Hero(Char):
    def __init__(self, dungeon, class_key: str, pos: int, abilities: dict | None = None) -> None:
        super().__init__(dungeon, pos)
        c = self.cls = HERO_CLASSES[class_key]
        self.class_key = class_key
        self.name = t(c["name"])
        self.abilities: dict[str, int] = dict(abilities or dice.DEFAULT_SCORES)
        self.lvl = 1
        self.exp = 0
        self.base_speed_ft = c.get("speed_ft", 30)
        # PV de nivel 1 en 5e: dado de golpe al máximo + modificador de CON
        self.HT = self.HP = max(1, c["hit_die"] + self.mod("CON"))
        self.awareness = 0.0
        self.update_awareness()
        self.hunger = 0.0
        self.second_wind = True  # Segundo Aliento disponible (se recarga con recover())
        self.slots: list[int] = spells.max_slots(self)  # espacios de conjuro disponibles por nivel
        self.free_casts: dict[str, int] = spells.max_free(self)  # Enemigo predilecto: Marca del cazador gratis
        self.marked = None  # quién tiene la Marca del cazador
        self.ranged: Item | None = None  # Hero.rangedWeapon: el arma del ataque a distancia en curso
        self.cover = 0  # cobertura del objetivo en el ataque a distancia en curso (Dungeon.shot): resta a la tirada
        # Belongings
        self.weapon: Item | None = self._starting(c.get("start_weapon"))
        self.armor: Item | None = self._starting(c.get("start_armor"))
        self.backpack: list[Item] = [Item(*i) if isinstance(i, (list, tuple)) else Item(i) for i in c.get("items", [])]
        for i in self.backpack:
            i.level_known = True  # el equipo inicial se conoce
        self.gold = 0
        self.quickslots: list = [c.get("quick"), None]  # acceso rápido: id (apilables) o el Item (equipo)
        dungeon.known.update(c.get("known", []))
        # turnos
        self.action: tuple | None = None   # ver docs/game/hero.md
        self.animating = False             # True tras moverse: el render lo baja al terminar la animación
        self.visible_enemies: list = []
        # varios héroes: piso propio, vista propia (Dungeon.observe) y actores que viajan con él entre pisos
        self.depth = 0
        self.fov: list[bool] = [False] * LENGTH
        self.ticks: list[Actor] = [Regeneration(self), Hunger(self)]
        # salvaciones contra la muerte (5e): [éxitos, fallos] mientras está caído a 0 PV
        self.dead = False
        self.saves = [0, 0]
        self.stable = False

    @staticmethod
    def _starting(item_id: str | None) -> Item | None:
        if item_id is None:
            return None
        item = Item(item_id)
        item.level_known = item.cursed_known = True
        return item

    # --- características (5e) ---
    def mod(self, ability: str) -> int:
        return dice.modifier(self.abilities[ability])

    @property
    def STR(self) -> int:
        return self.abilities["STR"]

    def proficiency(self) -> int:
        return dice.proficiency(self.lvl)

    def save_bonus(self, ability: str) -> int:
        return self.mod(ability) + (self.proficiency() if ability in self.cls.get("save_prof", []) else 0)

    def proficient_weapon(self, w: Item | None) -> bool:
        prof = self.cls["weapon_prof"]
        return w is None or w.id in prof or w.data["kind"] in prof  # sin arma (puños): siempre competente

    def proficient_armor(self, a: Item | None) -> bool:
        return a is None or a.data["kind"] == "clothing" or a.data["kind"] in self.cls["armor_prof"]

    # --- rasgos de clase 5e ---
    def recover(self) -> None:
        """Descanso largo (5e): Segundo Aliento, espacios de conjuro y usos gratis.
        ponytail: no hay descansos todavía; Dungeon.go_to lo llama al llegar a un piso nuevo."""
        self.second_wind = True
        self.slots = spells.max_slots(self)
        self.free_casts = spells.max_free(self)

    def has_trait(self, trait: str) -> bool:
        return trait in self.cls.get("traits", [])

    def sneak_dice(self) -> int:
        """Ataque Furtivo: 1d6 en nivel 1, +1d6 cada 2 niveles."""
        return (self.lvl + 1) // 2

    # --- combate 5e ---
    def _wep(self) -> Item | None:
        """El arma del ataque en curso: la arrojadiza / el arco mientras se dispara, si no la equipada."""
        return self.ranged or self.weapon

    def _attack_mod(self) -> int:
        w = self._wep()
        if w is not None and w.data.get("ranged") and self.ranged is not None:
            return self.mod("DEX")  # arco
        if w is not None and w.data.get("finesse"):
            return max(self.mod("STR"), self.mod("DEX"))
        return self.mod("STR")

    def armor_class(self) -> int:
        return armor_ac(self.armor, self.mod("DEX"))

    def attack_bonus(self, target) -> int:
        w = self._wep()
        bonus = self._attack_mod() + (self.proficiency() if self.proficient_weapon(w) else 0)
        return bonus + (w.level if w else 0) - self.str_missing(w) - self.cover

    def str_missing(self, item: Item | None) -> int:
        """Puntos de FUE que faltan para usar bien el ítem (0 si alcanza)."""
        return max(0, item.data["STR"] - self.STR) if item else 0

    def advantage(self, target) -> int:
        adv = super().advantage(target)
        heavy = self.str_missing(self._wep()) or self.str_missing(self.armor)
        far = self.ranged is not None and distance(self.pos, target.pos) > self.ranged_range()[0]
        # 5e: a distancia, desventaja si hay un enemigo al lado o si el objetivo está más allá del alcance normal
        close = self.ranged is not None and any(adjacent(self.pos, m.pos) for m in self.dungeon.mobs if m.is_alive())
        if heavy or not self.proficient_armor(self.armor) or far or close:
            adv -= 1  # ventaja y desventaja juntas se anulan
        return max(-1, min(1, adv))

    def roll_damage(self, crit: bool, adv: int = 0, target=None) -> tuple[int, str, tuple | None]:
        m = self._attack_mod()
        w = self._wep()
        if w is None:  # golpe sin armas: 1 + FUE
            return 1 + m, f"1{m:+d}", None
        rng = self.dungeon.rng
        melee_bow = w.data.get("ranged") and self.ranged is None  # el arco cuerpo a cuerpo: como un garrote
        total, text, rolled, sides = dice.roll(rng, "1d4" if melee_bow else weapon_dice(w), crit)
        n = len(rolled)
        label, hi = f"{n}d{sides}", n * sides
        # Ataque Furtivo: con ventaja y arma sutil o a distancia, dobles en crítico
        if adv > 0 and (w.data.get("finesse") or self.ranged is not None) and self.has_trait("pddnd:sneak_attack"):
            extra, etext, erolled, _ = dice.roll(rng, f"{self.sneak_dice()}d6", crit)
            total, text, rolled = total + extra, f"{text}+{etext}", rolled + erolled
            label, hi = f"{label}+{len(erolled)}d6", hi + 6 * len(erolled)
        if target is not None and target.buff("pddnd:hunters_mark"):  # Marca del cazador: +1d6 con armas
            extra, etext, erolled, _ = dice.roll(rng, "1d6", crit)
            total, text, rolled = total + extra, f"{text}+{etext}", rolled + erolled
            label, hi = f"{label}+{len(erolled)}d6", hi + 6 * len(erolled)
        return total + m, f"{text}{m:+d}", (label, sum(rolled), len(rolled), hi)

    def attack_delay(self) -> float:
        return self.weapon.data["dly"] if self.weapon else 1.0

    def speed(self) -> float:
        enc = self.armor.data["STR"] - self.STR if self.armor else 0  # del original: armadura muy pesada = lento
        return (self.base_speed / 1.3 ** enc if enc > 0 else self.base_speed) * self.buff_speed()

    def speed_ft(self) -> int:
        """5e: armadura pesada sin la FUE que pide = -10 ft."""
        a = self.armor
        heavy = a is not None and a.data.get("kind") == "heavy" and a.data["STR"] > self.STR
        return max(5, super().speed_ft() - (10 if heavy else 0))

    def initiative_bonus(self) -> int:
        return self.mod("DEX")

    def stealth(self) -> int:
        return max(0, self.mod("DEX"))  # más difícil que los mobs te noten

    def max_exp(self) -> int:
        return 5 + self.lvl * 5

    def update_awareness(self) -> None:
        self.awareness = 1 - self.cls["awareness_base"] ** ((1 + min(self.lvl, 9)) * 0.5)

    def earn_exp(self, exp: int) -> None:
        self.exp += exp
        while self.exp >= self.max_exp():
            self.exp -= self.max_exp()
            self.lvl += 1
            # 5e: promedio del dado de golpe + CON (mínimo 1) por nivel
            gain = max(1, self.cls["hit_die"] // 2 + 1 + self.mod("CON"))
            self.HT += gain
            if self.HP > 0:  # caído (XP repartida) no se levanta por subir de nivel
                self.HP += gain
            if self.lvl < 10:
                self.update_awareness()
            spells.level_up(self)
            self.dungeon.log(f"Subes a nivel {self.lvl}: +{gain} PV, competencia +{self.proficiency()}")
            self.dungeon.status(self, "sube de nivel", "pos")

    def is_starving(self) -> bool:
        return self.hunger >= STARVING

    # --- pertenencias ---
    def all_items(self) -> list[Item]:
        return [i for i in (self.weapon, self.armor) if i] + self.backpack

    def collect(self, item: Item) -> bool:
        """Mete un ítem en la mochila (apila si se puede). False si no hay lugar."""
        if item.category == "gold":
            self.gold += item.quantity
            return True
        if item.stackable():
            for i in self.backpack:
                if i.id == item.id:
                    i.quantity += item.quantity
                    return True
        if len(self.backpack) >= BACKPACK_SIZE:
            return False
        self.backpack.append(item)
        return True

    def quick_item(self, i: int) -> Item | None:
        """Ítem actual de la ranura rápida i (None si no hay o ya no se tiene)."""
        q = self.quickslots[i]
        if isinstance(q, str):
            return next((it for it in self.all_items() if it.id == q), None)
        return q if q is not None and q in self.all_items() else None

    def _take_one(self, item: Item) -> Item:
        """Saca una unidad del ítem de la mochila."""
        if item.quantity > 1:
            return item.split(1)
        self.backpack.remove(item)
        return item

    # --- turnos ---
    def waiting(self) -> bool:
        if not self.is_alive() or self.animating:
            return True
        # paralizado o caído: los turnos pasan solos
        return self.action is None and not self.has_flag("paralysed") and not self.is_downed()

    # --- a 0 PV: inconsciente y salvaciones contra la muerte (5e) ---
    def is_alive(self) -> bool:
        return not self.dead  # a 0 PV sigue vivo (caído) hasta fallar 3 salvaciones

    def is_downed(self) -> bool:
        return not self.dead and self.HP <= 0

    def heal(self, amount: int) -> int:
        """Cura hasta el máximo y devuelve lo curado. Cualquier curación levanta al caído."""
        amount = max(0, min(amount, self.HT - self.HP))
        was_down = self.is_downed()
        self.HP += amount
        if was_down and self.HP > 0:
            self.dungeon.log(f"{self.name} recupera la conciencia")
            self.dungeon.status(self, "se levanta", "pos")
        return amount

    def _death_save(self) -> None:
        """Al inicio de su turno: d20, 10+ éxito. 3 éxitos = estable, 3 fallos = muere, 20 = vuelve con 1 PV,
        1 = dos fallos. El resultado se aplica cuando el dado lo muestra (Dungeon.defer)."""
        d = self.dungeon
        nat, _ = dice.d20(d.rng)
        d.roll_event(t({"es": "Muerte CD10", "en": "Death DC10"}), nat, 1, 20, nat >= 10)

        def apply() -> None:
            if not self.is_downed() or self.stable:
                return
            if nat == 20:
                self.heal(1)
                return
            if nat >= 10:
                self.saves[0] += 1
            else:
                self.saves[1] += 2 if nat == 1 else 1
            s, f = self.saves
            d.log(f"{self.name}: salvación contra la muerte {nat} ({s} éxitos, {f} fallos)")
            if f >= 3:
                self.die(None)
            elif s >= 3:
                self.stable = True
                d.log(f"{self.name} está estable")
                d.status(self, "estable", "pos")

        d.defer(apply)

    def _stabilize(self, ally) -> None:
        """Medicina (5e): d20 + SAB contra CD 10 sobre un aliado caído al lado. Una acción.
        ponytail: sin competencia en habilidades todavía (Medicina sólo suma SAB)."""
        d = self.dungeon
        if not ally.is_downed() or ally.stable:
            return
        nat, _ = dice.d20(d.rng)
        bonus = self.mod("WIS")
        ok = nat + bonus >= 10
        d.roll_event(t({"es": "Medicina CD10", "en": "Medicine DC10"}), nat + bonus, 1 + bonus, 20 + bonus, ok)
        d.effect("anim", self, "operate", ally.pos)

        def apply() -> None:
            if ok and ally.is_downed():
                ally.stable = True
                d.log(f"{self.name} estabiliza a {ally.name}")
                d.status(ally, "estable", "pos")
            elif not ok:
                d.log(f"{self.name} no logra estabilizar a {ally.name}")

        d.defer(apply)
        self.spend(TIME_TO_REST)

    def handle(self, cell: int, stairs: bool = False) -> None:
        """Hero.handle: decide la acción según qué hay en la celda tocada/elegida.
        stairs = True sólo con click: cambiar de piso hay que confirmarlo; pisar la escalera no basta."""
        if not (0 <= cell < LENGTH):
            return
        level = self.dungeon.level
        ch = self.dungeon.find_char(cell)
        if self.fov[cell] and ch in self.dungeon.heroes and ch is not self:
            # aliado: si está caído se lo estabiliza; si no, se camina hacia él (no hay fuego amigo)
            self.action = ("help", ch) if ch.is_downed() else ("move", cell)
        elif self.fov[cell] and ch is not None and ch is not self:
            self.action = ("attack", ch)
        elif self.fov[cell] and cell in level.heaps:
            # cofre, tumba o restos: se abren desde al lado (OpenChest); un montón se recoge parado encima (PickUp)
            self.action = ("heap" if level.heaps[cell].type == HEAP else "open", cell)
        elif stairs and cell == level.exit:
            self.action = ("descend", cell)
        elif stairs and cell == level.entrance:
            self.action = ("ascend", cell)
        else:
            self.action = ("move", cell)

    def use(self, item: Item, verb: str, target: Item | None = None) -> None:
        """Acción sobre un ítem pedida desde la UI. Se ejecuta en el turno del héroe."""
        self.action = ("use", item, verb, target)

    def act(self) -> None:
        self.check_visible_mobs()
        if self.is_downed():
            self.action = None
            if not self.stable:
                self._death_save()
            self.spend(TIME_TO_REST)
            return
        if self.has_flag("paralysed"):
            self.action = None
            self.spend(TIME_TO_REST)
            return
        if self.action is None:  # interrumpido: no gasta tiempo, el planificador se detiene en waiting()
            return
        kind = self.action[0]
        if kind == "rest":
            self.action = None
            self.spend(TIME_TO_REST)
        elif kind == "search":
            self.action = None
            self.search(True)
            self.spend(TIME_TO_SEARCH)
        elif kind == "cast":  # ("cast", id de conjuro, celda): valida, lanza y gasta el tiempo (game/spells.py)
            _, sid, cell = self.action
            self.action = None
            spells.cast(self, sid, cell)
        elif kind == "second_wind":  # acción adicional: no gasta tiempo
            self.action = None
            self._second_wind()
        elif kind == "use":
            _, item, verb, target = self.action
            self.action = None
            self._use(item, verb, target)
        elif kind == "throw":  # ("throw", ítem, celda)
            _, item, cell = self.action
            self.action = None
            self._throw(item, cell)
        elif kind == "attack" and self._can_shoot(self.action[1]):  # con arco: dispara desde donde está
            enemy = self.action[1]
            self.action = None
            self._shoot(enemy)
        elif kind == "attack":
            enemy = self.action[1]
            if adjacent(self.pos, enemy.pos) and enemy.is_alive():
                self.action = None
                self.attack(enemy)
                self.spend(self.attack_delay())
            elif not (enemy.is_alive() and self.fov[enemy.pos] and self.get_closer(enemy.pos)):
                self.action = None
        elif kind == "help":  # ("help", aliado caído): acercarse y estabilizarlo
            ally = self.action[1]
            if adjacent(self.pos, ally.pos):
                self.action = None
                self._stabilize(ally)
            elif not (ally.is_downed() and self.get_closer(ally.pos)):
                self.action = None
        elif kind == "open":
            cell = self.action[1]
            if self.pos == cell or adjacent(self.pos, cell):
                self.action = None
                self._open(cell)
            elif not self.get_closer(cell):
                self.action = None
        elif kind in ("descend", "ascend", "heap") and self.pos == self.action[1]:
            self.action = None
            if kind == "descend":
                self.dungeon.descend(self)
            elif kind == "ascend":
                self.dungeon.ascend(self)
            else:
                self._pick_up()
        elif not self.get_closer(self.action[1]):  # "move", o "descend"/"ascend"/"heap" todavía lejos
            self.action = None

    # --- armas a distancia ---
    def ranged_range(self) -> tuple:
        """Alcance (normal, largo) en casillas del arma a distancia en curso."""
        return self.ranged.data.get("range", (1, 1)) if self.ranged else (1, 1)

    def _ammo(self) -> Item | None:
        w = self.weapon
        if not (w and w.data.get("ranged")):
            return None
        return next((i for i in self.backpack if i.id == w.data["ammo"]), None)

    def _can_shoot(self, enemy) -> bool:
        """Arco equipado, con flechas, enemigo a la vista, dentro del alcance largo y con línea de tiro."""
        d = self.dungeon
        if self._ammo() is None or not enemy.is_alive() or not self.fov[enemy.pos]:
            return False
        if distance(self.pos, enemy.pos) > self.weapon.data["range"][1]:
            return False
        return d.shot(self.pos, enemy.pos)[0] == enemy.pos

    def _ranged_attack(self, enemy, weapon: Item, cover: int = 0) -> bool:
        """Hero.shoot: ataque 5e con el arma a distancia (self.ranged y self.cover mientras dura)."""
        self.ranged, self.cover = weapon, cover
        if cover and self.dungeon.sees(enemy.pos):
            self.dungeon.log(f"{enemy.name} tiene media cobertura (-{cover})")
        try:
            return self.attack(enemy)
        finally:
            self.ranged, self.cover = None, 0

    def _after_shot(self, missile: Item, weapon: Item, enemy, hit: bool, stop: int) -> None:
        """Lo que pasa cuando la tirada ya se vio: estado del arma (on_hit), el proyectil cae, se pierde o vuelve."""
        d = self.dungeon

        def land() -> None:
            on_hit = weapon.data.get("on_hit")
            if hit and on_hit and enemy.is_alive():
                buff_id, duration, level, dc = on_hit
                enemy.add_buff(buff_id, duration, level, dc)
            if missile.data.get("returns"):  # el bumerán vuelve
                d.effect("missile", stop, self.pos, item_image(missile, d), missile.data.get("spin", 720))
                if not self.collect(missile):
                    d.level.drop(missile, self.pos)
            elif not hit:
                d.level.drop(missile, stop)

        if d.sees(stop):
            d.pending.append(land)
        else:
            land()

    def _throw(self, item: Item, cell: int) -> None:
        """Item.cast + MissileWeapon.onThrow: lanza una unidad. Si se frena en alguien, es un ataque a distancia."""
        d = self.dungeon
        if item not in self.backpack or not item.data.get("thrown") or cell == self.pos:
            return
        if not self.fov[cell]:
            d.log("No ves esa casilla")
            return
        stop, cover = d.shot(self.pos, cell)
        if stop == self.pos:
            return
        if distance(self.pos, stop) > item.data["range"][1]:
            d.log("Está demasiado lejos")
            return
        one = self._take_one(item)
        d.effect("anim", self, "attack", stop)
        d.effect("missile", self.pos, stop, item_image(one, d), one.data.get("spin", 720))
        enemy = d.find_char(stop)
        hit = enemy is not None and enemy not in d.heroes and self._ranged_attack(enemy, one, cover)  # a un aliado: cae ahí
        self._after_shot(one, one, enemy, hit, stop)
        self.spend(TIME_TO_THROW * one.data.get("dly", 1.0))

    def _shoot(self, enemy) -> None:
        """Dispara el arco: gasta una flecha (acertada se pierde; fallada cae junto al enemigo)."""
        d = self.dungeon
        arrow = self._take_one(self._ammo())
        d.effect("anim", self, "attack", enemy.pos)
        d.effect("missile", self.pos, enemy.pos, arrow.data.get("flight_image", item_image(arrow, d)), 0)
        hit = self._ranged_attack(enemy, self.weapon, d.shot(self.pos, enemy.pos)[1])
        self._after_shot(arrow, self.weapon, enemy, hit, enemy.pos)
        self.spend(self.weapon.data["dly"])

    def _second_wind(self) -> None:
        """Segundo Aliento (Guerrero): recupera 1d10 + nivel PV, una vez hasta recargarse."""
        d = self.dungeon
        if not self.has_trait("pddnd:second_wind"):
            return
        if not self.second_wind:
            d.log("Ya usaste el Segundo Aliento")
            return
        self.second_wind = False
        roll, _, rolled, _ = dice.roll(d.rng, "1d10")
        heal = self.heal(roll + self.lvl)
        d.roll_event("1d10", rolled[0], 1, 10)
        d.log(f"Segundo Aliento: +{heal} PV")
        d.status(self, f"+{heal}", "pos")

    def _open(self, cell: int) -> None:
        """Hero.actOpenChest + Heap.open: abre el cofre / tumba / restos desde al lado (o encima). Lo de adentro queda
        en el suelo como un montón normal: se recoge con click, caminando encima.
        ponytail: la tumba no invoca espectros ni los restos con ítem maldito hacen daño (faltan los Wraith)."""
        d = self.dungeon
        heap = d.level.heaps.get(cell)
        if heap is None or heap.type == HEAP:
            return
        msg = OPEN_MSG.get(heap.type, OPEN_MSG[CHEST])
        heap.type = HEAP
        if heap.items:
            d.log(t(msg))
        else:
            del d.level.heaps[cell]
            d.log(t({"es": "Está vacío", "en": "It's empty"}))
        d.effect("anim", self, "operate", cell)
        self.spend(TIME_TO_OPEN)

    def _pick_up(self) -> None:
        level = self.dungeon.level
        heap = level.heaps.get(self.pos)
        if heap is None:
            return
        if heap.type != HEAP:  # cofre, tumba o restos bajo los pies: primero se abren
            self._open(self.pos)
            return
        item = heap.items[0]
        if self.collect(item):
            heap.items.pop(0)
            if not heap.items:
                del level.heaps[self.pos]
            self.dungeon.log(f"Recoges {item.quantity} de oro" if item.category == "gold"
                             else f"Recoges {item_name(item, self.dungeon)}")
            self.spend(TIME_TO_PICK_UP)
        else:
            self.dungeon.log("Tu mochila está llena")

    def _use(self, item: Item, verb: str, target: Item | None) -> None:
        d = self.dungeon
        if item not in self.all_items():
            return
        cat = item.category
        if verb == "eat" and cat == "food":
            self._take_one(item)
            self.hunger = max(self.hunger - item.data["energy"], 0.0)
            d.log(f"Comes {t(item.data['name'])}")
            d.effect("anim", self, "operate", self.pos)
            self.spend(TIME_TO_EAT)
        elif verb in ("drink", "read") and cat in ("potion", "scroll"):
            need = item.data.get("target")
            if need and (target is None or target not in self.all_items() or target.id == item.id
                         or not TARGETS[need](target, d)):
                return
            self._take_one(item)
            d.known.add(item.id)
            EFFECTS[item.data["effect"]](self, item, target)
            d.effect("anim", self, "read" if verb == "read" else "operate", self.pos)
            self.spend(TIME_TO_USE)
        elif verb == "equip" and item.is_equipment():
            slot = "weapon" if cat == "weapon" else "armor"
            old = getattr(self, slot)
            if old and old.cursed:
                old.cursed_known = True
                d.log(f"No puedes quitarte {item_name(old, d)}: está maldito")
                return
            self.backpack.remove(item)
            if old:
                self.backpack.append(old)
            setattr(self, slot, item)
            item.level_known = item.cursed_known = True  # ponytail: en Java se identifica con el uso, no al equipar
            d.log(f"Te equipas {item_name(item, d)}" + (": ¡está maldito!" if item.cursed else ""))
            self.spend(TIME_TO_EQUIP * (2 if slot == "armor" else 1))
        elif verb == "unequip" and item in (self.weapon, self.armor):
            if item.cursed:
                item.cursed_known = True
                d.log(f"No puedes quitarte {item_name(item, d)}: está maldito")
            elif len(self.backpack) >= BACKPACK_SIZE:
                d.log("Tu mochila está llena")
            else:
                setattr(self, "weapon" if item is self.weapon else "armor", None)
                self.backpack.append(item)
                self.spend(TIME_TO_EQUIP)
        elif verb == "drop":
            if item in (self.weapon, self.armor):
                if item.cursed:
                    item.cursed_known = True
                    d.log(f"No puedes soltar {item_name(item, d)}: está maldito")
                    return
                setattr(self, "weapon" if item is self.weapon else "armor", None)
            else:
                self.backpack.remove(item)
            d.level.drop(item, self.pos)
            self.spend(TIME_TO_DROP)

    def interrupt(self) -> None:
        self.action = None

    def check_visible_mobs(self) -> None:
        """Si aparece un enemigo nuevo a la vista, se corta la acción en curso."""
        visible = [m for m in self.dungeon.mobs if self.fov[m.pos]]
        if any(m not in self.visible_enemies for m in visible):
            self.interrupt()
        self.visible_enemies = visible

    def get_closer(self, target: int) -> bool:
        level = self.dungeon.level
        if self.pos == target:
            return False
        if not self.can_step():
            return True  # sin movimiento en este turno de combate: sigue en el próximo
        if adjacent(self.pos, target):
            # ponytail: el abismo está bloqueado; en Java pide confirmación y caes al piso siguiente
            free = self.dungeon.find_char(target) is None and not level.pit[target]
            step = target if free and (level.passable[target] or level.avoid[target]) else -1
        else:
            # sólo camina por lo ya visto (visited/mapped)
            passable = [p and (v or m) for p, v, m in zip(level.passable, level.visited, level.mapped)]
            step = self.dungeon.find_path(self, self.pos, target, passable, self.fov)
        if step == -1:
            return False
        self.move(step)
        self.spend(1 / self.speed())
        self.animating = True
        self.dungeon.observe()
        self.search(False)
        return True

    def search(self, intentional: bool) -> None:
        """Busca secretos en las 8 celdas vecinas visibles. Pasiva (tras cada paso) con prob. awareness."""
        d = self.dungeon
        level, rng = d.level, d.rng
        found = False
        for n in NEIGHBOURS9:
            p = self.pos + n
            if not self.fov[p]:
                continue
            if intentional:
                d.effect("checked", p)  # CheckedCell: destello azul en cada casilla revisada
            if level.secret[p] and (intentional or rng.Float() < self.awareness):
                level.discover(p)
                found = True
        if intentional:
            d.status(self, "buscando")
            d.effect("anim", self, "operate", self.pos)
        if found:
            d.log("Encuentras algo oculto")
            d.observe()
        elif intentional:
            d.log("Buscas, pero no encuentras nada")

    def damage(self, dmg: int, src) -> None:
        """5e: a 0 PV cae inconsciente (salvo daño masivo: lo que sobra >= PV máximos = muere). Caído, cada daño es
        un fallo de salvación, dos si el atacante está al lado (en 5e ese golpe es crítico), y deja de estar estable."""
        self.interrupt()
        if self.dead:
            return
        d = self.dungeon
        if self.HP <= 0:
            # daño por turno (Buff): según la regla de la partida "dot_death_saves"
            if dmg > 0 and (d.rules["dot_death_saves"] or not isinstance(src, Buff)):
                n = 2 if isinstance(src, Char) and adjacent(src.pos, self.pos) else 1
                self.stable = False
                self.saves[1] += n
                d.status(self, "fallo", "neg")
                what = getattr(src, "name", None) or "daño"
                d.log(f"{self.name} recibe daño estando caído ({what}): {n} fallo{'s' if n > 1 else ''} "
                      f"({min(self.saves[1], 3)}/3)")
                if self.saves[1] >= 3:
                    self.die(src)
            return
        if dmg > 0:
            d.status(self, str(dmg), "neg")
        self.HP -= dmg
        if self.HP <= 0:
            massive = -self.HP >= self.HT
            self.HP = 0
            if massive:
                self.die(src)
                return
            self.saves, self.stable, self.action = [0, 0], False, None
            d.log(f"{self.name} cae inconsciente")
            d.status(self, "inconsciente", "neg")

    def die(self, src) -> None:
        """Sale del planificador con lo suyo: el piso sigue para los demás héroes; sin héroes vivos, nada avanza.
        ponytail: el muerto queda de espectador; volver a entrar con un personaje nuevo llega con la red (pasos 5-6)."""
        d = self.dungeon
        self.dead, self.HP = True, 0
        for a in (self, *self.ticks, *self.buffs):
            d.scheduler.remove(a)
        cause = f" ({src.name})" if getattr(src, "name", None) else ""
        if len(d.heroes) == 1:
            d.log(f"Has muerto{cause}. R = nueva partida")
        else:
            d.log(f"{self.name} muere{cause}")


class Regeneration(Actor):
    """buffs/Regeneration.java: +1 HP cada 10 de tiempo, salvo famélico."""

    def __init__(self, hero: Hero) -> None:
        super().__init__()
        self.hero = hero

    def act(self) -> None:
        h = self.hero
        # caído: sólo si está estable (en 5e recupera 1 PV con el tiempo y despierta)
        if h.is_alive() and h.HP < h.HT and not h.is_starving() and (h.HP > 0 or h.stable):
            h.heal(1)
        self.spend(REGENERATION_DELAY)


class Hunger(Actor):
    """buffs/Hunger.java: +10 de hambre cada 10 de tiempo (Rogue cada 12). Hambriento a 260, famélico a 360:
    ahí deja de regenerar y pierde 1 HP con 30% de probabilidad en cada paso. El valor vive en hero.hunger."""

    name = "hambre"

    def __init__(self, hero: Hero) -> None:
        super().__init__()
        self.hero = hero

    def act(self) -> None:
        h, d = self.hero, self.hero.dungeon
        if h.is_alive():
            if h.is_starving():
                if d.rng.Float() < 0.3:  # se puede morir de hambre (como en Java, sin parálisis)
                    h.damage(1, self)
            else:
                new = h.hunger + HUNGER_STEP
                if new >= STARVING:
                    d.log("¡Te mueres de hambre!")
                    h.interrupt()
                elif new >= HUNGRY > h.hunger:
                    d.log("Tienes hambre")
                h.hunger = min(new, STARVING)
        self.spend(h.cls.get("hunger_step", HUNGER_STEP))
