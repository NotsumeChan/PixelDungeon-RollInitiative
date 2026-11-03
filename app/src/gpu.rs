//! Todo el dibujo por GPU (wgpu). Dos pasadas por frame:
//! 1. sprites -> textura de resolución virtual (~320x180, pixeles del juego): un draw call con instancias;
//! 2. esa textura -> pantalla, escalada con factor entero y NEAREST (pixeles nítidos, sin bordes borrosos).
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use winit::window::Window;

use crate::atlas::Atlas;

/// Alto y ancho de referencia del juego (el original): el factor de zoom es el entero que más se acerca.
const BASE_W: u32 = 320;
const BASE_H: u32 = 180;
const VIRT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
const BACKGROUND: wgpu::Color = wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };

/// Una instancia del lote: un rectángulo del atlas dibujado en `pos` (pixeles del juego), teñido por `color`.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Sprite {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub uv: [f32; 4], // u0, v0, u1, v1
    pub color: [f32; 4],
}

struct Virtual {
    w: u32,
    h: u32,
    zoom: u32,
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
    sprite_group: wgpu::BindGroup,
    globals: wgpu::Buffer,
    instances: wgpu::Buffer,
    blit_pipe: wgpu::RenderPipeline,
    blit_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    virt: Virtual,
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
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs"),
            compilation_options: Default::default(),
            buffers,
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
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

        // --- atlas ---
        let extent = wgpu::Extent3d { width: atlas.w, height: atlas.h, depth_or_array_layers: 1 };
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("atlas"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: VIRT_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            &atlas.rgba,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * atlas.w), rows_per_image: Some(atlas.h) },
            extent,
        );
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // --- pasada 1: sprites ---
        let globals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
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
        let atlas_view = tex.create_view(&Default::default());
        let sprite_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &sprite_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: globals.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&atlas_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });
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
            VIRT_FORMAT,
        );
        let instances = Self::instance_buffer(&device, 1024);

        // --- pasada 2: textura virtual -> pantalla ---
        let blit_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[entry(0, TEXTURE), entry(1, SAMPLER)],
        });
        let blit_shader = device.create_shader_module(wgpu::include_wgsl!("blit.wgsl"));
        let blit_pipe = pipeline(&device, &blit_shader, &blit_layout, &[], config.format);

        let virt = Self::make_virtual(&device, &blit_layout, &sampler, config.width, config.height);
        Self {
            window,
            surface,
            device,
            queue,
            config,
            sprite_pipe,
            sprite_group,
            globals,
            instances,
            blit_pipe,
            blit_layout,
            sampler,
            virt,
        }
    }

    fn instance_buffer(device: &wgpu::Device, n: usize) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sprites"),
            size: (n * size_of::<Sprite>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    /// Resolución virtual para una ventana: zoom entero (el mayor que deja ~320x180 a la vista) y lo que entra
    /// de mundo con ese zoom. Igual que `core/Screen.py`: en una pantalla ancha se ve más mapa, no se estira.
    fn make_virtual(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, sampler: &wgpu::Sampler, w: u32, h: u32) -> Virtual {
        let zoom = (w / BASE_W).min(h / BASE_H).max(1);
        let (vw, vh) = ((w / zoom).max(1), (h / zoom).max(1));
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("virtual"),
            size: wgpu::Extent3d { width: vw, height: vh, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: VIRT_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = tex.create_view(&Default::default());
        let blit_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(sampler) },
            ],
        });
        Virtual { w: vw, h: vh, zoom, view, blit_group }
    }

    pub fn virtual_size(&self) -> (u32, u32) {
        (self.virt.w, self.virt.h)
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        if w == 0 || h == 0 {
            return; // minimizada
        }
        self.config.width = w;
        self.config.height = h;
        self.surface.configure(&self.device, &self.config);
        self.virt = Self::make_virtual(&self.device, &self.blit_layout, &self.sampler, w, h);
    }

    pub fn draw(&mut self, sprites: &[Sprite]) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f) | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            _ => return, // timeout u oculta: se salta el frame
        };
        let needed = (sprites.len() * size_of::<Sprite>()) as u64;
        if needed > self.instances.size() {
            self.instances = Self::instance_buffer(&self.device, sprites.len().next_power_of_two());
        }
        self.queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(sprites));
        self.queue.write_buffer(&self.globals, 0, bytemuck::cast_slice(&[self.virt.w as f32, self.virt.h as f32, 0.0, 0.0]));

        let screen = frame.texture.create_view(&Default::default());
        let mut enc = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.virt.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(BACKGROUND), store: wgpu::StoreOp::Store },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.sprite_pipe);
            pass.set_bind_group(0, &self.sprite_group, &[]);
            pass.set_vertex_buffer(0, self.instances.slice(..));
            pass.draw(0..6, 0..sprites.len() as u32);
        }
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &screen,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(BACKGROUND), store: wgpu::StoreOp::Store },
                })],
                ..Default::default()
            });
            let (w, h) = (self.virt.w * self.virt.zoom, self.virt.h * self.virt.zoom);
            let (x, y) = ((self.config.width - w) / 2, (self.config.height - h) / 2); // centrado
            pass.set_viewport(x as f32, y as f32, w as f32, h as f32, 0.0, 1.0);
            pass.set_pipeline(&self.blit_pipe);
            pass.set_bind_group(0, &self.virt.blit_group, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit([enc.finish()]);
        self.queue.present(frame);
    }
}
