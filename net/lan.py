"""Red LAN (fase 5, paso 4), sólo stdlib. Descubrimiento por broadcast UDP y conexión TCP con asyncio en un hilo
aparte. El juego habla con la red sólo por colas (inbox) y por send(): el hilo de red nunca toca pygame ni game/.
Mensajes: un JSON por línea, siempre un dict con "t" (tipo). Sin pickle: un mensaje nunca ejecuta código."""
import asyncio
import hashlib
import json
import queue
import socket
import threading
import time

from game import registry

UDP_PORT, TCP_PORT = 47810, 47811
ANNOUNCE_EVERY = 1.0  # s entre anuncios del host
GAME_TTL = 3.0        # s sin anuncio = la partida ya no aparece en la lista
MAX_LINE = 8 * 1024 * 1024  # un estado completo (save.dump) entra holgado; más largo = se corta la conexión
MAGIC = "pddnd"
NET_EVENTS = ("join", "leave")  # los genera la red, no el otro lado


def content_hash() -> str:
    """Hash del contenido (los data/*.json de cada carpeta de registros, en orden): si no coincide, no se entra."""
    h = hashlib.sha256()
    for folder in registry.DATA_DIRS:
        for p in sorted(folder.glob("*.json")):
            h.update(p.name.encode())
            h.update(p.read_bytes())
    return h.hexdigest()[:16]


def _encode(msg: dict) -> bytes:
    return (json.dumps(msg, ensure_ascii=False, separators=(",", ":")) + "\n").encode()


def _decode(line: bytes) -> dict | None:
    try:
        msg = json.loads(line)
    except (ValueError, UnicodeDecodeError):
        return None
    return msg if isinstance(msg, dict) and isinstance(msg.get("t"), str) else None


class _Loop:
    """Un hilo con su propio loop de asyncio."""

    def __init__(self) -> None:
        self.loop = asyncio.new_event_loop()
        threading.Thread(target=self.loop.run_forever, daemon=True).start()

    def run(self, coro):
        return asyncio.run_coroutine_threadsafe(coro, self.loop)

    def call(self, fn, *args) -> None:
        self.loop.call_soon_threadsafe(fn, *args)

    def stop(self) -> None:
        self.loop.call_soon_threadsafe(self.loop.stop)


class Host:
    """Crea la partida: acepta clientes por TCP y se anuncia por UDP. inbox: (id de cliente, mensaje). Al conectarse
    un cliente llega {"t": "join"} y al irse {"t": "leave"}."""

    def __init__(self, name: str, version: str, port: int = TCP_PORT) -> None:
        self.inbox: queue.Queue = queue.Queue()
        self.info = {"name": name, "players": 1, "version": version, "hash": content_hash(), "port": port}
        self._writers: dict = {}
        self._next = 1
        self._net = _Loop()
        try:
            self._server = self._net.run(asyncio.start_server(self._client, "0.0.0.0", port, limit=MAX_LINE)).result(5)
        except BaseException:
            self._net.stop()
            raise  # puerto ocupado u otro error de red: lo maneja la UI
        self._announcer = self._net.run(self._announce())

    def set_info(self, **kw) -> None:
        """Cambia lo que se anuncia (p. ej. players). Se reemplaza el dict entero: el hilo de red nunca ve uno a medias."""
        self.info = {**self.info, **kw}

    async def _client(self, reader, writer) -> None:
        cid, self._next = self._next, self._next + 1
        self._writers[cid] = writer
        self.inbox.put((cid, {"t": "join"}))
        try:
            while line := await reader.readline():
                msg = _decode(line)
                if msg is None or msg["t"] in NET_EVENTS:
                    break  # basura o se hace pasar por la red: se corta
                self.inbox.put((cid, msg))
        except (ConnectionError, ValueError):  # ValueError = línea más larga que MAX_LINE
            pass
        finally:
            self._writers.pop(cid, None)
            writer.close()
            self.inbox.put((cid, {"t": "leave"}))

    async def _announce(self) -> None:
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
        s.setblocking(False)
        try:
            while True:
                try:
                    s.sendto(json.dumps({"magic": MAGIC, **self.info}).encode(), ("<broadcast>", UDP_PORT))
                except OSError:
                    pass  # sin red por ahora: se reintenta
                await asyncio.sleep(ANNOUNCE_EVERY)
        finally:
            s.close()

    # --- desde el hilo del juego ---
    def send(self, cid: int, msg: dict) -> None:
        self._net.call(self._write, cid, _encode(msg))

    def broadcast(self, msg: dict) -> None:
        data = _encode(msg)
        self._net.call(lambda: [w.write(data) for w in self._writers.values()])

    def kick(self, cid: int) -> None:
        self._net.call(self._close_one, cid)

    def close(self) -> None:
        self._announcer.cancel()
        self._net.call(self._shutdown)
        self._net.stop()

    # --- en el hilo de red ---
    def _write(self, cid: int, data: bytes) -> None:
        # ponytail: sin drain(); un cliente que no lee acumula buffer. Con 2-4 jugadores en LAN alcanza
        w = self._writers.get(cid)
        if w is not None:
            w.write(data)

    def _close_one(self, cid: int) -> None:
        w = self._writers.get(cid)
        if w is not None:
            w.close()

    def _shutdown(self) -> None:
        self._server.close()
        for w in list(self._writers.values()):
            w.close()


class Client:
    """Se une a una partida (ip, puerto). inbox: los mensajes del host; {"t": "leave"} si se corta la conexión.
    Si no puede conectar, el constructor lanza OSError / TimeoutError (lo maneja la UI)."""

    def __init__(self, host: str, port: int = TCP_PORT) -> None:
        self.inbox: queue.Queue = queue.Queue()
        self._net = _Loop()
        try:
            self._reader, self._writer = self._net.run(
                asyncio.open_connection(host, port, limit=MAX_LINE)).result(5)
        except BaseException:
            self._net.stop()
            raise
        self._net.run(self._read())

    async def _read(self) -> None:
        try:
            while line := await self._reader.readline():
                msg = _decode(line)
                if msg is not None and msg["t"] not in NET_EVENTS:
                    self.inbox.put(msg)
        except (ConnectionError, ValueError):
            pass
        finally:
            self.inbox.put({"t": "leave"})

    def send(self, msg: dict) -> None:
        self._net.call(self._writer.write, _encode(msg))

    def close(self) -> None:
        self._net.call(self._writer.close)
        self._net.stop()


class Browser:
    """Escucha los anuncios UDP de los hosts. games(): [(ip, info)] vistos en los últimos GAME_TTL segundos."""

    def __init__(self) -> None:
        self._seen: dict = {}
        self._closed = False
        self._sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self._sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        if hasattr(socket, "SO_REUSEPORT"):  # varias instancias en la misma PC (pruebas)
            self._sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEPORT, 1)
        self._sock.bind(("", UDP_PORT))
        self._sock.settimeout(0.5)
        threading.Thread(target=self._listen, daemon=True).start()

    def _listen(self) -> None:
        while not self._closed:
            try:
                data, (ip, _) = self._sock.recvfrom(4096)
            except socket.timeout:
                continue
            except OSError:
                return  # socket cerrado
            try:
                info = json.loads(data)
            except (ValueError, UnicodeDecodeError):
                continue
            if isinstance(info, dict) and info.get("magic") == MAGIC and isinstance(info.get("port"), int):
                self._seen[(ip, info["port"])] = (time.monotonic(), info)

    def games(self) -> list[tuple[str, dict]]:
        now = time.monotonic()
        return [(ip, info) for (ip, _), (seen, info) in list(self._seen.items()) if now - seen < GAME_TTL]

    def close(self) -> None:
        self._closed = True
        self._sock.close()
