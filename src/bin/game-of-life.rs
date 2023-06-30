use log::{info, debug};
use wgpu::RequestAdapterOptions;
use winit::{event_loop::{self, EventLoop, ControlFlow}, event::{Event::WindowEvent, KeyboardInput, VirtualKeyCode, ElementState}, dpi::LogicalSize};

async fn run() {
    println!("hello world");

    pretty_env_logger::init();
    info!("Starting");

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
    });
    
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Game of Life")
        .with_inner_size(LogicalSize::new(800, 600))
        .build(&event_loop).expect("Failed to create window");

    // # Safety: window must live at least as long as surface
    let surface = unsafe { instance.create_surface(&window) }.expect("Failed to create surface");

    let adapter = instance.request_adapter(&RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }).await.expect("Failed to request adapter");

    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor { 
        label: Some("Device"), limits: Default::default(),
        features: Default::default()
    }, None).await.expect("Failed to request device");

    let adapter_info = adapter.get_info();
    debug!("Using adapter: {:?}", adapter_info);

    event_loop.run(move |event, _, control_flow| {
        match event {
            WindowEvent { window_id, event } if window_id == window.id() => {
                match event {
                    winit::event::WindowEvent::KeyboardInput { input, ..} => {
                        handle_keyboard_input(input, control_flow);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
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
            *control_flow  = ControlFlow::Exit;
        }
        _ => {}
    }
}
