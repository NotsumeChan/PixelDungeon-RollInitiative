//! Todo el dibujo por GPU (wgpu). Tres pasadas por frame, todas con lotes de instancias:
//! 1. mundo -> textura del mundo (pixeles del mundo; su propio zoom entero, el de la cámara);
//! 2. UI -> textura de la UI (resolución virtual ~320x180, zoom entero de la pantalla), transparente;
//! 3. las dos texturas -> pantalla, cada una escalada con su factor entero y NEAREST, la UI encima.
use std::ops::Range;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use winit::window::Window;

use crate::atlas::Atlas;

/// Resolución de referencia del original: el zoom de la UI es el mayor entero que deja al menos esto a la vista.
const BASE_W: u32 = 320;
const BASE_H: u32 = 180;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
const BLACK: wgpu::Color = wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
const CLEAR: wgpu::Color = wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
pub const FOG_SIZE: u32 = 33; // una esquina por celda: (32 + 1) x (32 + 1)

/// Una instancia del lote: un rectángulo de una textura dibujado en `pos` (pixeles de su capa), teñido por `color`.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Sprite {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub uv: [f32; 4], // u0, v0, u1, v1
    pub color: [f32; 4],
}

/// Con qué textura se dibuja un tramo del lote del mundo.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tex {
    Atlas,
    Fog, // filtro LINEAR: el degradado entre esquinas de FogOfWar
}

/// Una capa (mundo o UI): su textura, su tamaño en pixeles propios y su factor entero hacia la pantalla.
struct Target {
    w: u32,
    h: u32,
    scale: u32,
    offset: (u32, u32),
    view: wgpu::TextureView,
    blit_group: wgpu::BindGroup,
}

pub struct Gpu {
    pub window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    sprite_pipe: wgpu::RenderPipeline,
    blit_pipe: wgpu::RenderPipeline,
    blit_layout: wgpu::BindGroupLayout,
    nearest: wgpu::Sampler,
    instances: wgpu::Buffer,
    world_globals: wgpu::Buffer,
    ui_globals: wgpu::Buffer,
    atlas_world: wgpu::BindGroup,
    fog_world: wgpu::BindGroup,
    atlas_ui: wgpu::BindGroup,
    fog: wgpu::Texture,
    world: Target,
    ui: Target,
}

fn entry(binding: u32, ty: wgpu::BindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry { binding, visibility: wgpu::ShaderStages::VERTEX_FRAGMENT, ty, count: None }
}

const TEXTURE: wgpu::BindingType = wgpu::BindingType::Texture {
    sample_type: wgpu::TextureSampleType::Float { filterable: true },
    view_dimension: wgpu::TextureViewDimension::D2,
    multisampled: false,
};
const SAMPLER: wgpu::BindingType = wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering);

fn pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    layout: &wgpu::BindGroupLayout,
    buffers: &[Option<wgpu::VertexBufferLayout>],
    format: wgpu::TextureFormat,
    blend: wgpu::BlendState,
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&layout),
        vertex: wgpu::VertexState { module: shader, entry_point: Some("vs"), compilation_options: Default::default(), buffers },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState { format, blend: Some(blend), write_mask: wgpu::ColorWrites::ALL })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

fn texture(device: &wgpu::Device, w: u32, h: u32, usage: wgpu::TextureUsages) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage,
        view_formats: &[],
    })
}

fn uniform(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn attach(view: &wgpu::TextureView, clear: wgpu::Color) -> [Option<wgpu::RenderPassColorAttachment<'_>>; 1] {
    [Some(wgpu::RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations { load: wgpu::LoadOp::Clear(clear), store: wgpu::StoreOp::Store },
    })]
}

/// Zoom entero de la UI para una ventana (como `core/Screen.py`).
fn ui_zoom(w: u32, h: u32) -> u32 {
    (w / BASE_W).min(h / BASE_H).max(1)
}

impl Gpu {
    pub async fn new(window: Arc<Window>, atlas: &Atlas) -> Self {
        // display handle: lo pide el backend GLES (EGL en Wayland/X11 y Android)
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(Box::new(window.clone())));
        let surface = instance.create_surface(window.clone()).expect("superficie");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("no hay GPU compatible");
        log::info!("GPU: {:?}", adapter.get_info());
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor { required_limits: adapter.limits(), ..Default::default() })
            .await
            .expect("dispositivo");

        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let mut config = surface.get_default_config(&adapter, size.width.max(1), size.height.max(1)).expect("formato");
        config.format = caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(caps.formats[0]);
        // Sin vsync si se puede (para medir los 200+ FPS). Android suele ofrecer sólo Fifo.
        config.present_mode = [wgpu::PresentMode::Immediate, wgpu::PresentMode::Mailbox]
            .into_iter()
            .find(|m| caps.present_modes.contains(m))
            .unwrap_or(wgpu::PresentMode::Fifo);
        log::info!("superficie {:?} {:?}", config.format, config.present_mode);
        surface.configure(&device, &config);

        // --- texturas: atlas (NEAREST) y niebla (LINEAR) ---
        let copy = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
        let atlas_tex = texture(&device, atlas.w, atlas.h, copy);
        Self::upload(&queue, &atlas_tex, atlas.w, atlas.h, &atlas.rgba);
        let fog = texture(&device, FOG_SIZE, FOG_SIZE, copy);
        let nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let linear = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // --- lote de sprites: un bind group por (capa, textura) ---
        let sprite_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                entry(0, wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                }),
                entry(1, TEXTURE),
                entry(2, SAMPLER),
            ],
        });
        let group = |globals: &wgpu::Buffer, tex: &wgpu::Texture, sampler: &wgpu::Sampler| {
            let view = tex.create_view(&Default::default());
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &sprite_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: globals.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&view) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(sampler) },
                ],
            })
        };
        let (world_globals, ui_globals) = (uniform(&device), uniform(&device));
        let atlas_world = group(&world_globals, &atlas_tex, &nearest);
        let fog_world = group(&world_globals, &fog, &linear);
        let atlas_ui = group(&ui_globals, &atlas_tex, &nearest);

        let sprite_shader = device.create_shader_module(wgpu::include_wgsl!("sprite.wgsl"));
        let sprite_pipe = pipeline(
            &device,
            &sprite_shader,
            &sprite_layout,
            &[Some(wgpu::VertexBufferLayout {
                array_stride: size_of::<Sprite>() as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4, 3 => Float32x4],
            })],
            FORMAT,
            wgpu::BlendState::ALPHA_BLENDING,
        );
        let instances = Self::instance_buffer(&device, 4096);

        // --- composición: cada capa a la pantalla (premultiplicado: la UI se dibujó sobre transparente) ---
        let blit_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[entry(0, TEXTURE), entry(1, SAMPLER)],
        });
        let blit_shader = device.create_shader_module(wgpu::include_wgsl!("blit.wgsl"));
        let blit_pipe = pipeline(
            &device,
            &blit_shader,
            &blit_layout,
            &[],
            config.format,
            wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
        );

        let z = ui_zoom(config.width, config.height);
        let ui = Self::target(&device, &blit_layout, &nearest, config.width, config.height, z);
        let world = Self::target(&device, &blit_layout, &nearest, config.width, config.height, z);
        Self {
            window,
            surface,
            device,
            queue,
            config,
            sprite_pipe,
            blit_pipe,
            blit_layout,
            nearest,
            instances,
            world_globals,
            ui_globals,
            atlas_world,
            fog_world,
            atlas_ui,
            fog,
            world,
            ui,
        }
    }

    fn upload(queue: &wgpu::Queue, tex: &wgpu::Texture, w: u32, h: u32, rgba: &[u8]) {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            rgba,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * w), rows_per_image: Some(h) },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
    }

    fn instance_buffer(device: &wgpu::Device, n: usize) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sprites"),
            size: (n * size_of::<Sprite>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    /// Capa del tamaño de la ventana dividida por `scale` (lo que entra; en una pantalla ancha se ve más), centrada.
    fn target(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, sampler: &wgpu::Sampler, w: u32, h: u32, scale: u32) -> Target {
        let (tw, th) = ((w / scale).max(1), (h / scale).max(1));
        let tex = texture(device, tw, th, wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING);
        let view = tex.create_view(&Default::default());
        let blit_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(sampler) },
            ],
        });
        let offset = ((w - tw * scale) / 2, (h - th * scale) / 2);
        Target { w: tw, h: th, scale, offset, view, blit_group }
    }

    /// Zoom entero de la UI (pantalla / resolución virtual): base para los zooms de la cámara.
    pub fn ui_zoom(&self) -> u32 {
        self.ui.scale
    }

    /// Tamaño de la UI en pixeles virtuales.
    pub fn ui_size(&self) -> (u32, u32) {
        (self.ui.w, self.ui.h)
    }

    /// Tamaño del mundo visible (pixeles del mundo), dónde empieza en la ventana y su factor.
    pub fn world_view(&self) -> (u32, u32, (u32, u32), u32) {
        (self.world.w, self.world.h, self.world.offset, self.world.scale)
    }

    /// Cambia el zoom de la cámara (pixeles de pantalla por pixel del mundo, entero).
    pub fn set_world_scale(&mut self, scale: u32) {
        if scale != self.world.scale {
            self.world = Self::target(&self.device, &self.blit_layout, &self.nearest, self.config.width, self.config.height, scale);
        }
    }

    /// Esquinas de la niebla: FOG_SIZE x FOG_SIZE RGBA.
    pub fn set_fog(&self, rgba: &[u8]) {
        Self::upload(&self.queue, &self.fog, FOG_SIZE, FOG_SIZE, rgba);
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        if w == 0 || h == 0 {
            return; // minimizada
        }
        self.config.width = w;
        self.config.height = h;
        self.surface.configure(&self.device, &self.config);
        self.ui = Self::target(&self.device, &self.blit_layout, &self.nearest, w, h, ui_zoom(w, h));
        self.world = Self::target(&self.device, &self.blit_layout, &self.nearest, w, h, self.world.scale);
    }

    /// `sprites` lleva el mundo y la UI juntos: `world` son tramos (textura, rango) en orden; `ui` usa el atlas.
    pub fn draw(&mut self, sprites: &[Sprite], world: &[(Tex, Range<u32>)], ui: Range<u32>) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f) | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            _ => return, // timeout u oculta: se salta el frame
        };
        if (sprites.len() * size_of::<Sprite>()) as u64 > self.instances.size() {
            self.instances = Self::instance_buffer(&self.device, sprites.len().next_power_of_two());
        }
        self.queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(sprites));
        let size = |t: &Target| [t.w as f32, t.h as f32, 0.0, 0.0];
        self.queue.write_buffer(&self.world_globals, 0, bytemuck::cast_slice(&size(&self.world)));
        self.queue.write_buffer(&self.ui_globals, 0, bytemuck::cast_slice(&size(&self.ui)));

        let screen = frame.texture.create_view(&Default::default());
        let mut enc = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &attach(&self.world.view, BLACK),
                ..Default::default()
            });
            pass.set_pipeline(&self.sprite_pipe);
            pass.set_vertex_buffer(0, self.instances.slice(..));
            for (tex, range) in world {
                pass.set_bind_group(0, if *tex == Tex::Fog { &self.fog_world } else { &self.atlas_world }, &[]);
                pass.draw(0..6, range.clone());
            }
        }
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &attach(&self.ui.view, CLEAR),
                ..Default::default()
            });
            pass.set_pipeline(&self.sprite_pipe);
            pass.set_vertex_buffer(0, self.instances.slice(..));
            pass.set_bind_group(0, &self.atlas_ui, &[]);
            pass.draw(0..6, ui);
        }
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &attach(&screen, BLACK),
                ..Default::default()
            });
            pass.set_pipeline(&self.blit_pipe);
            for t in [&self.world, &self.ui] {
                let (x, y) = t.offset;
                pass.set_viewport(x as f32, y as f32, (t.w * t.scale) as f32, (t.h * t.scale) as f32, 0.0, 1.0);
                pass.set_bind_group(0, &t.blit_group, &[]);
                pass.draw(0..3, 0..1);
            }
        }
        self.queue.submit([enc.finish()]);
        self.queue.present(frame);
    }
}
