use log::{info, warn};
use rugu::State;
use winit::{event::{Event::WindowEvent, KeyboardInput, VirtualKeyCode, ElementState}, event_loop::ControlFlow};

async fn run() {
    println!("hello world");

    pretty_env_logger::init();
    info!("Starting");

    let (mut state, event_loop) = State::new().await;

    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::MainEventsCleared => {
            state.window.request_redraw();
        }
        winit::event::Event::RedrawRequested(id) if id == state.window.id() => {
            match state.draw() {
                Ok(()) => {}
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(wgpu::SurfaceError::Lost) => {
                    state.resize(state.size);
                }
                // should be resolved by the next frame
                Err(err @ wgpu::SurfaceError::Timeout | err @ wgpu::SurfaceError::Outdated) => warn!("Err: {:?}", err),
            }
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
