"""actors/Actor.java: planificador de turnos por tiempo, y rondas de combate 5e por iniciativa."""

TICK = 1.0
CALM_ROUNDS = 2  # rondas seguidas sin ningún mob cazando a la vista para salir del combate


class Actor:
    in_rounds = False  # True = en combate actúa por iniciativa (personajes); False = sigue el tiempo (estados, timers)

    def __init__(self) -> None:
        self.time = 0.0

    def act(self) -> None:
        """Hace su turno y DEBE gastar tiempo con spend(), o el planificador lo vuelve a elegir."""
        raise NotImplementedError

    def spend(self, t: float) -> None:
        self.time += t

    def waiting(self) -> bool:
        """True = el planificador se detiene en este actor (el héroe esperando input o su animación)."""
        return False

    def fighting(self) -> bool:
        """True = mete al piso en combate (un mob que caza a un héroe que ve)."""
        return False


class Scheduler:
    """Fuera de combate, Actor.process: siempre actúa el de menor `time`; en empate, el agregado primero.
    En combate (order != None), rondas 5e: cada personaje, por iniciativa, hace su turno (movimiento + 1 acción,
    ver Char.start_turn); al final de la ronda los estados y timers avanzan un TICK.
    Uno por piso, no global (varios pisos / jugadores)."""

    def __init__(self) -> None:
        self.actors: list[Actor] = []
        self.now = 0.0
        self.order: list | None = None  # combate: personajes por iniciativa (None = fuera de combate)
        self.turn = 0                   # índice en order de a quién le toca
        self.round = 0
        self.calm = 0                   # rondas seguidas sin nadie peleando

    def add(self, actor: Actor, delay: float = 0.0) -> None:
        actor.time = self.now + delay
        self.actors.append(actor)

    def remove(self, actor: Actor) -> None:
        if actor in self.actors:
            self.actors.remove(actor)

    def current(self):
        """A quién le toca: en combate, al de la iniciativa; si no, al de menor time."""
        if self.order is not None:
            return self.order[self.turn] if self.turn < len(self.order) else None
        return min(self.actors, key=lambda a: a.time, default=None)

    def process(self, stop=None) -> None:
        """Corre turnos hasta que el próximo actor esté esperando (o hasta que stop() sea True)."""
        while self.actors:
            if stop is not None and stop():
                return
            if self.order is not None:
                if not self._combat_step():
                    return
                continue
            current = self.current()
            if current.waiting():
                return
            self.now = current.time
            current.act()
            if any(a.fighting() for a in self.actors):
                self.start_combat()

    # --- combate 5e ---
    def start_combat(self) -> None:
        """Todos los personajes del piso tiran iniciativa y se ordenan (en empate, el agregado primero)."""
        chars = [a for a in self.actors if a.in_rounds and a.is_alive()]
        for a in chars:
            a.roll_initiative()
        self.order = sorted(chars, key=lambda a: -a.init)
        self.turn, self.round, self.calm = 0, 1, 0
        d = self.order[0].dungeon
        seen = [f"{a.name} {a.init}" for a in self.order if d.level.visible[a.pos]]
        d.log("¡Combate! Iniciativa: " + ", ".join(seen))
        self.order[0].start_turn()

    def _combat_step(self) -> bool:
        """Un paso de la ronda. False = hay que esperar (al héroe al que le toca)."""
        if self.turn >= len(self.order):
            self._end_round()
            return True
        c = self.order[self.turn]
        if c in self.actors and not c.turn_over():
            if c.waiting():
                return False
            c.act()
            return True
        self.turn += 1
        if self.turn < len(self.order):
            self.order[self.turn].start_turn()
        return True

    def _end_round(self) -> None:
        """Fin de ronda: estados y timers avanzan un TICK (1 ronda = 1 turno de PD); entran los personajes nuevos
        (respawn, héroe que baja) con su iniciativa; CALM_ROUNDS rondas sin nadie peleando = fin del combate."""
        end = self.now + TICK
        while True:
            due = [a for a in self.actors if not a.in_rounds and a.time <= end]
            if not due:
                break
            a = min(due, key=lambda a: a.time)
            self.now = max(self.now, a.time)
            a.act()
        self.now = end
        order = [a for a in self.order if a in self.actors]
        new = [a for a in self.actors if a.in_rounds and a not in order and a.is_alive()]
        for a in new:
            a.roll_initiative()
        # ponytail: el que entra a mitad de combate actúa desde la ronda siguiente (5e: desde su iniciativa)
        self.order = sorted(order + new, key=lambda a: -a.init)
        self.calm = 0 if any(a.fighting() for a in self.actors) else self.calm + 1
        if self.calm >= CALM_ROUNDS or not self.order:
            for a in self.order:
                a.time = self.now  # vuelven al tiempo de PD todos parejos
            if self.order:
                self.order[0].dungeon.log("Fin del combate")
            self.order = None
            return
        self.turn = 0
        self.round += 1
        self.order[0].start_turn()
