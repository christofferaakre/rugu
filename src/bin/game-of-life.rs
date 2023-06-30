use std::time::Instant;

use log::{debug, info};
use wgpu::{
    include_wgsl, Adapter, ColorTargetState, Device, FrontFace, PipelineLayout, PrimitiveState,
    Queue, RenderPipeline, RequestAdapterOptions, ShaderModule, Surface, TextureViewDimension,
};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event::WindowEvent, KeyboardInput, VirtualKeyCode},
    event_loop::{self, ControlFlow, EventLoop},
    window::Window,
};

pub struct State {
    counter: Instant,
    pub window: Window,
    pub surface: Surface,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub shader_module: ShaderModule,
    pub pipeline_layout: PipelineLayout,
    pub render_pipeline: RenderPipeline,
}

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
            Err(err) => todo!("Need to handle lost and outdated surface errors; recreate surface"),
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
            render_pass.draw(0..1, 0..1);
        }

        self.queue.submit(std::iter::once(command_encoder.finish()));
        surface_texture.present();
    }
}

async fn init() -> (State, EventLoop<()>) {
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
    let surface = unsafe { instance.create_surface(&window) }.expect("Failed to create surface");

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

    let shader_module = device.create_shader_module(include_wgsl!("../../shaders/shader.wgsl"));

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
            buffers: &[],
        },
        primitive: PrimitiveState::default(),
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

    let state = State {
        counter: Instant::now(),
        window,
        surface,
        adapter,
        device,
        queue,
        shader_module,
        pipeline_layout,
        render_pipeline,
    };

    (state, event_loop)
}

async fn run() {
    println!("hello world");

    pretty_env_logger::init();
    info!("Starting");

    let (mut state, event_loop) = init().await;

    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::MainEventsCleared => {
            state.window.request_redraw();
        }
        winit::event::Event::RedrawRequested(id) if id == state.window.id() => {
            state.draw();
        }
        WindowEvent { window_id, event } if window_id == state.window.id() => match event {
            winit::event::WindowEvent::KeyboardInput { input, .. } => {
                handle_keyboard_input(input, control_flow);
            }
            _ => {}
        },
        _ => {}
    });
}

fn main() {
    pollster::block_on(run());
}

pub fn handle_keyboard_input(input: KeyboardInput, control_flow: &mut ControlFlow) {
    if input.virtual_keycode.is_none() {
        return;
    }
    let code = input.virtual_keycode.unwrap();
    match code {
        VirtualKeyCode::Escape if input.state == ElementState::Pressed => {
            *control_flow = ControlFlow::Exit;
        }
        _ => {}
    }
}
