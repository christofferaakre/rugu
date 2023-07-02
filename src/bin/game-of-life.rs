#![allow(clippy::single_match)]
#![allow(clippy::collapsible_match)]



use log::{info};
use rugu::State;

use winit::{
    event::{ElementState, Event::WindowEvent, KeyboardInput, VirtualKeyCode},
    event_loop::{ControlFlow},
};


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
