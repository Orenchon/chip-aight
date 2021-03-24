mod components;
use components::cpu::Cpu;
use components::sound::SoundManager;
use pixels::{Error, Pixels, SurfaceTexture};
use std::thread::sleep;
use std::time::{Duration, Instant};
use winit::event::{ElementState, StartCause, VirtualKeyCode};
use winit::{
    dpi::LogicalSize,
    event::{Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
mod dumb_tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
static KEY_MAP: [VirtualKeyCode; 16] = [
    VirtualKeyCode::Key1,
    VirtualKeyCode::Key2,
    VirtualKeyCode::Key3,
    VirtualKeyCode::Key4,
    VirtualKeyCode::Q,
    VirtualKeyCode::W,
    VirtualKeyCode::E,
    VirtualKeyCode::R,
    VirtualKeyCode::A,
    VirtualKeyCode::S,
    VirtualKeyCode::D,
    VirtualKeyCode::F,
    VirtualKeyCode::Z,
    VirtualKeyCode::X,
    VirtualKeyCode::C,
    VirtualKeyCode::V,
];
fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_inner_size(LogicalSize::new(640, 320));
    let mut is_key_pressed: [bool; 16] = [false; 16];
    let last_frame = 0;
    let size = window.inner_size();
    let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
    let mut state: [[bool; 32]; 64] = [[false; 32]; 64];
    let mut pixels = Pixels::new(64, 32, surface_texture).unwrap();
    let mut last_draw = Instant::now();
    let mut sound_system = SoundManager::new().unwrap();
    //sound_system.play();
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => *control_flow = ControlFlow::Exit,
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(virtual_code),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                let key_pressed = KEY_MAP.iter().position(|&s| s == virtual_code);
                match key_pressed {
                    Some(key) => {
                        //println!("{:?}", state);
                        is_key_pressed[key] = true;
                    }
                    _ => (),
                }
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(virtual_code),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } => {
                let key_pressed = KEY_MAP.iter().position(|&s| s == virtual_code);
                match key_pressed {
                    Some(key) => {
                        //println!("{:?}", state);
                        is_key_pressed[key] = false;
                    }
                    _ => (),
                }
            }
            _ => (),
        },
        Event::MainEventsCleared => {
            window.request_redraw();
            *control_flow = ControlFlow::Poll
        }
        Event::RedrawRequested(_window_id) => {
            // Draw it to the `SurfaceTexture`
            //if last_draw.elapsed().as_millis() > 16 {
            let frame = pixels.get_frame();
            /*for (idx, pixel) in frame.chunks_exact_mut(512).enumerate() {
                for mini_pix in pixel {
                    if is_key_pressed[idx] {
                        *mini_pix = 0xFF
                    } else {
                        *mini_pix = 0x00
                    }
                }
            }*/
            //let chunks = frame.chunks_exact_mut(10);
            let chunks = frame.chunks_exact_mut(4);
            /*for (row, row_key) in is_key_pressed.chunks_exact_mut(4).enumerate() {
                for (col, sub_key) in row_key.iter().enumerate() {
                    if *sub_key {
                        state[col][row] = true
                    } else {
                        state[col][row] = false
                    }
                }
            }
            Test input for display
            */
            println!("{}", chunks.len());
            // Draw chunks to the screen
            for (idx, pixel) in chunks.enumerate() {
                let row = idx / 64;
                let col = idx % 64;
                if row < state[0].len() {
                    for rgba_value in pixel {
                        if state[col][row] {
                            *rgba_value = 0xFF
                        } else {
                            *rgba_value = 0x00
                        }
                    }
                }
            }
            pixels.render().unwrap();
            //}
            last_draw = Instant::now()
        }
        Event::RedrawEventsCleared => {}
        _ => (),
    });
}
