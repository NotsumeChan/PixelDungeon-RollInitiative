"""Partida en red (fase 5, paso 4): pega lan.Host / lan.Client a una Dungeon. El host simula (autoritativo); el
cliente manda su input y recibe el estado completo (save.dump) para dibujarlo. pump() va una vez por frame, en el
hilo del juego (nunca en el de red)."""
import json
import queue

from game import save, dice
from game.hero import HERO_CLASSES
from . import actions, lan

NAME_MAX = 16


class HostGame:
    """Lado host: hello -> se suma un héroe a la partida; action -> se aplica a SU héroe; leave -> su héroe sale.
    Tras cada pump, si el estado cambió, se le manda a cada cliente con "you" = su índice en d.heroes."""

    def __init__(self, dungeon, host: "lan.Host") -> None:
        self.d, self.host = dungeon, host
        self.players: dict = {}  # id de cliente -> Hero
        self._last = None        # último estado mandado (sin la hora), para no repetirlo

    def _refuse(self, msg: dict) -> str | None:
        if msg.get("version") != self.host.info["version"]:
            return "version distinta"
        if msg.get("hash") != self.host.info["hash"]:
            return "contenido distinto (data/ o mods)"
        ab = msg.get("abilities")
        if msg.get("class") not in HERO_CLASSES or not isinstance(ab, dict) or not dice.valid_scores(ab):
            return "personaje invalido"
        return None

    def pump(self) -> None:
        d = self.d
        while True:
            try:
                cid, msg = self.host.inbox.get_nowait()
            except queue.Empty:
                break
            kind = msg["t"]
            if kind == "hello" and cid not in self.players:
                why = self._refuse(msg)
                if why:
                    self.host.send(cid, {"t": "refused", "why": why})
                    self.host.kick(cid)
                    continue
                # ponytail: el personaje entra nuevo (nivel 1, equipo inicial); el personaje portátil es el paso 5
                hero = d.add_hero(msg["class"], {a: msg["abilities"][a] for a in dice.ABILITIES})
                hero.name = str(msg.get("name") or hero.name)[:NAME_MAX]
                d.enter(d.hero.depth)
                self.players[cid] = hero
                d.log(f"{hero.name} se une a la partida")
                self.host.set_info(players=1 + len(self.players))
                self._last = None  # que el nuevo reciba el estado ya
            elif kind == "action" and cid in self.players:
                hero = self.players[cid]
                d.enter(hero.depth)  # handle() mira el piso en contexto: el de ESE héroe
                actions.apply(hero, msg)
                d.enter(d.hero.depth)
            elif kind == "leave":
                hero = self.players.pop(cid, None)
                if hero is not None:
                    d.remove_hero(hero)
                    d.log(f"{hero.name} se va de la partida")
                    self.host.set_info(players=1 + len(self.players))
        if self.players:
            data = save.dump(d)
            key = json.dumps({**data, "saved": None}, sort_keys=True)
            if key != self._last:
                # ponytail: estado completo en cada cambio; con mapas grandes o internet, mandar sólo diferencias
                self._last = key
                for cid, hero in self.players.items():
                    self.host.send(cid, {"t": "state", "you": d.heroes.index(hero), "save": data})

    def close(self) -> None:
        self.host.close()


class ClientGame:
    """Lado cliente: manda hello al conectar. pump() devuelve una Dungeon armada con el último estado recibido (o None
    si no llegó nada nuevo). refused = motivo si el host lo rechazó; closed = se cortó la conexión."""

    def __init__(self, client: "lan.Client", version: str, name: str, class_key: str, abilities: dict) -> None:
        self.client = client
        self.refused: str | None = None
        self.closed = False
        client.send({"t": "hello", "version": version, "hash": lan.content_hash(), "name": name,
                     "class": class_key, "abilities": abilities})

    def send(self, msg: dict) -> None:
        """msg: uno de net.actions (handle, simple, use, throw, cast)."""
        self.client.send(msg)

    def pump(self):
        latest = None
        while True:
            try:
                msg = self.client.inbox.get_nowait()
            except queue.Empty:
                break
            if msg["t"] == "state":
                latest = msg
            elif msg["t"] == "refused":
                self.refused = str(msg.get("why"))
            elif msg["t"] == "leave":
                self.closed = True
        if latest is None:
            return None
        # ponytail: se rearma la Dungeon entera (regenera los pisos con la seed) en cada estado. El paso 6 lo aplica
        # sobre la existente, con ids estables para que los sprites no salten
        try:
            return save.load({**latest["save"], "local": latest["you"]})
        except (ValueError, KeyError, TypeError, IndexError):
            return None  # estado roto: se espera al siguiente

    def close(self) -> None:
        self.client.close()
