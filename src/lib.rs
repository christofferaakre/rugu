use std::time::Instant;

use log::{debug, warn};
use wgpu::{
    include_wgsl, util::DeviceExt, Adapter, ColorTargetState, Device, PipelineLayout,
    PrimitiveState, Queue, RenderPipeline, RequestAdapterOptions, ShaderModule, Surface,
    TextureViewDimension, VertexBufferLayout,
};
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event_loop::EventLoop,
    window::Window,
};

pub struct State {
    pub window: Window,
    pub size: winit::dpi::PhysicalSize<u32>,
    counter: Instant,
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    shader_module: ShaderModule,
    pipeline_layout: PipelineLayout,
    render_pipeline: RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    config: wgpu::SurfaceConfiguration,
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
}

pub const TRIANGLE_VERTICES: [Vertex; 3] = [
    Vertex {
        position: [-0.5, -0.5, 0.0],
    },
    Vertex {
        position: [0.0, 0.5, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
    },
];

impl State {
    pub async fn new() -> (Self, EventLoop<()>) {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let event_loop = EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_title("Game of Life")
            .with_inner_size(LogicalSize::new(800, 600))
            .build(&event_loop)
            .expect("Failed to create window");

        let size = window.inner_size();

        // # Safety: window must live at least as long as surface
        let surface =
            unsafe { instance.create_surface(&window) }.expect("Failed to create surface");

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to request adapter");

        let adapter_info = adapter.get_info();
        debug!("Using adapter: {:?}", adapter_info);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    limits: Default::default(),
                    features: Default::default(),
                },
                None,
            )
            .await
            .expect("Failed to request device");

        let surface_caps = surface.get_capabilities(&adapter);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
            format: *surface_caps
                .formats
                .get(0)
                .expect("Surface had no supported texture formats"),
            width: size.width,
            height: size.height,
            present_mode: *surface_caps
                .present_modes
                .get(0)
                .unwrap_or(&wgpu::PresentMode::Fifo),
            alpha_mode: *surface_caps
                .alpha_modes
                .get(0)
                .unwrap_or(&wgpu::CompositeAlphaMode::default()),
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let shader_module = device.create_shader_module(include_wgsl!("../shaders/shader.wgsl"));

        let vertex_buffer_layout = VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position ([f32; 3])
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
            ],
        };

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(&TRIANGLE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[vertex_buffer_layout],
            },
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: surface.get_current_texture().unwrap().texture.format(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let state = Self {
            counter: Instant::now(),
            window,
            surface,
            adapter,
            device,
            queue,
            shader_module,
            pipeline_layout,
            render_pipeline,
            vertex_buffer,
            size,
            config
        };

        (state, event_loop)
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.height > 0 && new_size.width > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn draw(&mut self) -> Result<(), wgpu::SurfaceError> {
        let dt = self.counter.elapsed();
        self.counter = Instant::now();
        let fps = 1.0 / dt.as_secs_f32();
        debug!("{fps:02} fps");

        let surface_texture = self.surface.get_current_texture()?;

        let current_texture = &surface_texture.texture;

        let view = current_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Output texture view"),
            format: Some(current_texture.format()),
            dimension: Some(TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let mut command_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render command encoder"),
                });
        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..TRIANGLE_VERTICES.len() as u32, 0..1);
        }

        self.queue.submit(std::iter::once(command_encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}
