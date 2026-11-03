# app/src/gpu.rs

Todo el dibujo por GPU con wgpu 30. Reemplaza `core/Screen.py` del prototipo (que componía en la CPU con pygame y
subía el frame entero como textura).

- **Pasada 1 (sprites)**: los `Sprite` del frame (posición y tamaño en pixeles del juego, recorte del atlas en uv,
  color que tiñe) van a un buffer de instancias y se dibujan en **un draw call** (`draw(0..6, 0..n)`): el vertex shader
  arma el quad con `vertex_index`. Se dibuja sobre una textura de resolución virtual.
- **Pasada 2 (blit)**: la textura virtual a la pantalla con un triángulo que cubre el viewport; el viewport mide la
  resolución virtual por el zoom entero y va centrado. Sampler `NEAREST`: pixeles nítidos.
- Resolución virtual (`make_virtual`): zoom = el mayor entero que deja al menos 320x180; la resolución virtual es lo
  que entra con ese zoom (en una pantalla ancha se ve más mundo, no se estira). Como `core/Screen.py`.
- Colores: atlas, textura virtual y superficie en sRGB (sin conversiones raras de color entre pasadas).
- Present mode: `Immediate` > `Mailbox` > `Fifo`. Android suele dar sólo `Fifo`: ahí el FPS es el del panel.
- El buffer de instancias crece a la potencia de 2 siguiente si un frame trae más sprites.
- `CurrentSurfaceTexture::Outdated/Lost` reconfigura la superficie y salta el frame.
- wgpu 30: `Instance::new` por valor y con display handle (`new_with_display_handle`), vertex buffers como `Option`, `present` está en `Queue`, `PipelineLayoutDescriptor` lleva
  `immediate_size`, `RenderPipelineDescriptor` lleva `multiview_mask`.
