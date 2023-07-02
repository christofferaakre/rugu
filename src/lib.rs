#![allow(clippy::collapsible_match)]
#![allow(clippy::single_match)]

use std::time::Instant;

use cgmath::{SquareMatrix, Vector2, Vector3};
use log::{debug, warn};
use wgpu::{
    include_wgsl, util::DeviceExt, Adapter, ColorTargetState, Device, PipelineLayout,
    PrimitiveState, Queue, RenderPipeline, RequestAdapterOptions, ShaderModule, Surface,
    TextureViewDimension, VertexBufferLayout,
};
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::Window};

pub struct State {
    pub window: Window,
    counter: Instant,
    surface: Surface,
    _adapter: Adapter,
    device: Device,
    queue: Queue,
    _shader_module: ShaderModule,
    _pipeline_layout: PipelineLayout,
    render_pipeline: RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    model_bind_group: wgpu::BindGroup,
    instance_buffer: wgpu::Buffer,
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

pub const SQUARE_VERTICES: [Vertex; 4] = [
    Vertex {
        position: [-0.5, -0.5, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.0],
    },
];

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
struct InstanceRaw {
    position: [[f32; 4]; 4],
}

struct Instance {
    position: Vector2<f32>,
}

impl From<Instance> for InstanceRaw {
    fn from(instance: Instance) -> Self {
        Self {
            position: cgmath::Matrix4::from_translation(Vector3::new(
                instance.position.x,
                instance.position.y,
                0.0,
            ))
            .into(),
        }
    }
}

// const INSTANCE_DATA: [Instance; 2] = [
//     Instance {
//         position: Vector2::new(-0.5, -0.5),
//     },
//     Instance {
//         position: Vector2::new(0.5, 0.6),
//     },
// ];

const INSTANCE_DATA: [Instance; 1] = [Instance {
    position: Vector2::new(0.0, 0.0),
}];

impl State {
    pub fn draw(&mut self) {
        let dt = self.counter.elapsed();
        self.counter = Instant::now();
        let fps = 1.0 / dt.as_secs_f32();
        debug!("{fps:02} fps");

        let surface_texture = match self.surface.get_current_texture() {
            Ok(surface_texture) => surface_texture,
            Err(wgpu::SurfaceError::Timeout) => {
                return;
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                panic!("Out of memory!")
            }
            // Err(_err) => todo!("Need to handle lost and outdated surface errors; recreate surface"),
            Err(err) => {
                warn!("Error: {:?}", err);
                return;
            }
        };

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
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_bind_group(0, &self.model_bind_group, &[]);
            render_pass.draw(
                0..SQUARE_VERTICES.len() as u32,
                0..INSTANCE_DATA.len() as u32,
            );
        }

        self.queue.submit(std::iter::once(command_encoder.finish()));
        surface_texture.present();
    }
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

        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
                format: *surface_caps
                    .formats
                    .get(0)
                    .expect("Surface had no supported texture formats"),
                width: window.inner_size().width,
                height: window.inner_size().height,
                present_mode: *surface_caps
                    .present_modes
                    .get(0)
                    .unwrap_or(&wgpu::PresentMode::Fifo),
                alpha_mode: *surface_caps
                    .alpha_modes
                    .get(0)
                    .unwrap_or(&wgpu::CompositeAlphaMode::default()),
                view_formats: vec![],
            },
        );

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
            contents: bytemuck::cast_slice(&SQUARE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let instance_data = INSTANCE_DATA.map(InstanceRaw::from);

        let instance_buffer_layout = VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 2 * std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 3 * std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 4,
                },
            ],
        };

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // let identity_matrix = cgmath::Matrix4::<f32>::identity();
        let model_matrix = cgmath::Matrix4::from_translation(cgmath::Vector3::new(-0.5, 0.0, 0.0));
        let model: [[f32; 4]; 4] = model_matrix.into();

        let model_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(&model),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let model_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Model matrix bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Model matrix bind group"),
            layout: &model_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: model_uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render pipeline layout"),
            bind_group_layouts: &[&model_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[vertex_buffer_layout, instance_buffer_layout],
            },
            primitive: PrimitiveState { 
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: surface.get_current_texture().unwrap().texture.format(),
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let state = Self {
            counter: Instant::now(),
            window,
            surface,
            _adapter: adapter,
            device,
            queue,
            _shader_module: shader_module,
            _pipeline_layout: pipeline_layout,
            render_pipeline,
            vertex_buffer,
            model_bind_group,
            instance_buffer,
        };

        (state, event_loop)
    }
}
