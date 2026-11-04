# app/src/gpu.rs

Todo el dibujo por GPU con wgpu 30. Reemplaza `core/Screen.py` del prototipo (que componía en la CPU con pygame).

Tres pasadas por frame:
1. **Mundo** -> textura del mundo, en pixeles del mundo. Tramos del lote `(Tex, rango)`: el atlas con NEAREST y la
   niebla con LINEAR (otro bind group, mismo pipeline).
2. **UI** -> textura de la UI en resolución virtual (~320x180), sobre transparente.
3. **Composición**: las dos texturas a la pantalla, cada una escalada con su factor entero y centrada (viewport), la
   UI encima con blending premultiplicado (se dibujó sobre transparente).

- Zoom de la UI (`ui_zoom`): el mayor entero que deja al menos 320x180 (180x320 en vertical), como `core/Screen.py`. El de la cámara
  (`set_world_scale`) lo elige `lib.rs`; el mundo visible es lo que entra con ese factor.
- `world_view()`: tamaño del mundo visible, su posición en la ventana y su factor (para pasar del mouse a la celda).
- `set_fog(rgba)`: sube las esquinas de la niebla (33x33) cuando cambia la vista.
- Colores: atlas, capas y superficie en sRGB.
- Present mode: `Immediate` > `Mailbox` > `Fifo`. Android suele dar sólo `Fifo`: ahí el FPS es el del panel.
- El buffer de instancias crece a la potencia de 2 siguiente si un frame trae más sprites.
- `CurrentSurfaceTexture::Outdated/Lost` reconfigura la superficie y salta el frame.
- wgpu 30: `Instance::new` por valor y con display handle, vertex buffers como `Option`, `present` está en `Queue`,
  `PipelineLayoutDescriptor` lleva `immediate_size`, `RenderPipelineDescriptor` lleva `multiview_mask`.

`ui_view()`: dónde empieza la capa de UI en la ventana y su factor, para pasar el mouse / toque a pixeles de la
UI.
