# Decisiones del dueño

No se vuelven a discutir salvo que el dueño lo pida.

## Meta: un juego propio (2026-11-06)
- A largo plazo, **separarse por completo de Pixel Dungeon**: mobs e ítems del SRD 5.2 (CC-BY 4.0), generación,
  código y UI propios, arte y sonido hechos por personas. Caro y lento, sin apuro: lo nuevo nace propio y lo viejo se
  reemplaza de a poco. Pasos en [`pendientes.md`](pendientes.md#separarse-de-pixel-dungeon).
- **Nombre:** sigue siendo PixelDungeon-RollInitiative mientras quede algo de PD. El nuevo se elige al separarse
  (no "Roll Initiative" solo: es marca registrada).
- **Rama `pd-core`:** antes del primer cambio que reemplace algo de PD, se crea desde `main` y queda con el core de PD
  para quien le interese.
- Libre y gratis siempre, y el agradecimiento a Pixel Dungeon queda en los créditos aunque ya no sea obligatorio.
- Mientras quede algo de PD, el juego es GPL v3; al separarse, la licencia del código nuevo se decide entonces.

## Distribución y red (2026-11-06)
- **Releases:** cada merge a `main` publica `v<versión>` con Linux, Windows y el APK firmado. → [D1](pasos/1-releases.md)
- **Datos al desinstalar:** Android con `hasFragileUserData` (casilla "Conservar datos"). Linux y Windows ya los
  conservan. Instalador de Windows, después.
- **Internet:** dirección a mano (sirve para playit.gg, ngrok, Tailscale; ZeroTier ya anda) + contraseña opcional.
  Relay y servidor dedicado, después. → [D2](pasos/2-red-internet.md)
- **Compatibilidad:** por versión de protocolo + hash del contenido. El cliente actualiza a la versión **del host**,
  no a la última. Android no baja de versión. → [D3](pasos/3-compatibilidad.md)
- **Actualizador:** feature `updater`, sólo builds de GitHub o propias (nunca Play ni Steam). Avisa y descarga, sólo
  estables, apagable en Configuración (prendido por defecto). → [D4](pasos/4-updater.md)
- **Steam:** primero sin SDK (como Shattered PD). Amigos y logros: a futuro, sin implementar (revisar licencia).
- **Play Store:** sólo publicar, sin Play Games; la clave del dueño en Play App Signing.
- **Orden:** releases → red → compatibilidad → actualizador → mapas y editor.

## Mapas y editor (2026-11-06)
Editor como aplicación aparte (sólo PC). Primero mapas prefabricados, con el mundo por zonas desde el inicio. Después
del actualizador. → [M](pasos/5-mapas-editor.md)

## NPCs (2026-11-05)
1. Contenido inicial: Fabricante de varitas y Mercader; el Fantasma, no.
2. Misiones, flags y actitud son de la partida (compartidas en la LAN); las recompensas, del héroe.
3. Tiradas sociales con pericias 5e.
4. Atacar a un NPC lo vuelve hostil y los que lo ven te tratan peor.

## Mods
- Sin tienda: se consiguen por fuera. Packs estilo Modrinth; al unirse, los mods del host estilo tModLoader.
- Obligatorios (se compara el hash) y de cliente (gráficos, sonido).
- Mods con código (Lua): advertencia fuerte y confirmación. Sólo datos: aviso general.
- Plataformas: Android, Windows, Linux. Sin iOS ni Mac.

## Multijugador (2026-11-04)
1. Host autoritativo.
2. Combate por iniciativa 5e; fuera de combate, simultáneo.
3. Cada jugador puede estar en un piso distinto.
4. Salvaciones contra la muerte; si muere, vuelve con un personaje nuevo.
5. El botín es de quien lo agarra; XP repartida con bonus al último golpe.
6. Cada jugador guarda su personaje (estilo Terraria); el host guarda la partida.
7. Reglas de la partida opcionales (daño estando caído, "mitad si supera").

## Proyecto (2026-11-03)
- **Rust** en vez de Python: todo por GPU (wgpu + winit, sin SDL), 200+ FPS, Linux, Windows y Android. El prototipo
  queda congelado en `python-prototipe`.
- Dos crates: `game/` (lógica sin render) y `app/` (ventana, render, UI, red con `std::net` e hilos).
- Determinismo: `Rng` compatible con CPython, nada de `HashMap` que decida resultados.
- Pruebas a mano; tests automáticos sólo de red. Guardados en formato nuevo, sin migración.
- APK firmado con clave propia fuera del repo (`~/.pddnd-signing/`). Licencia GPL v3.
- 5e manda sobre los stats de PD ([`dnd5e.md`](dnd5e.md)).
