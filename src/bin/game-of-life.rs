use log::info;
use winit::{event_loop::{self, EventLoop, ControlFlow}, event::{Event::WindowEvent, KeyboardInput, VirtualKeyCode, ElementState}, dpi::LogicalSize};

fn main() {
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
    let surface = unsafe { instance.create_surface(&window) };

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
