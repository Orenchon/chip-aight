mod components;
use components::memory::Memory;
use components::sound::SoundManager;
use components::{cpu::Cpu, sound};
use getopts::Options;
use pixels::{Error, Pixels, SurfaceTexture};
use std::env;
use std::fs;
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
/*static KEY_MAP: [VirtualKeyCode; 16] = [
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
];*/
static KEY_MAP: [VirtualKeyCode; 16] = [
    VirtualKeyCode::Key1, // 1
    VirtualKeyCode::Key2, // 2
    VirtualKeyCode::Key3, // 3
    VirtualKeyCode::Q,    // 4
    VirtualKeyCode::W,    // 5
    VirtualKeyCode::E,    // 6
    VirtualKeyCode::A,    // 7
    VirtualKeyCode::S,    // 8
    VirtualKeyCode::D,    // 9
    VirtualKeyCode::Z,    // A
    VirtualKeyCode::X,    // 0
    VirtualKeyCode::C,    // B
    VirtualKeyCode::Key4, // C
    VirtualKeyCode::R,    // D
    VirtualKeyCode::F,    // E
    VirtualKeyCode::V,    // F
];

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optopt("h", "hertz", "Custom cpu operations per second", "INT");
    opts.optflag(
        "",
        "store-load-quirks",
        "Used to not change the value of I in Fx55 and Fx65",
    );
    opts.optflag("", "shift-y", "Used to use y as a base in shift functions");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!(f.to_string())
        }
    };
    let mut hz: u128 = 500;
    hz = match matches.opt_str("hertz") {
        Some(hertz) => hertz.parse::<u128>().expect("hz is not a valid number"),
        _ => hz,
    };

    let one_cycle_time: u128 = 1000000 / hz;
    //let one_cycle_time: u128 = 1000000;
    let filename = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };
    let file = load_from_file(&filename);
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_inner_size(LogicalSize::new(640, 320));
    let mut mem = Memory {
        ..Default::default()
    };
    let mut cpu = Cpu {
        ..Default::default()
    };
    cpu.store_load_quirk = matches.opt_present("store-load-quirks");
    cpu.shift_y = matches.opt_present("shift-y");
    mem.load(&file).expect("Couldn't load program to memory");
    Cpu::write_fonts_to_mem(&mut mem);
    //mem.print_memory();
    let mut is_key_pressed: [bool; 16] = [false; 16];
    let last_frame = 0;
    let size = window.inner_size();
    let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
    let mut state: [[bool; 32]; 64] = [[false; 32]; 64];
    let mut pixels = Pixels::new(64, 32, surface_texture).unwrap();
    let mut last_draw = Instant::now();
    let mut last_cpu = Instant::now();
    let mut sound_system = SoundManager::new().unwrap();
    let mut keep_trying = true;
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
                        is_key_pressed[key] = false;
                    }
                    _ => (),
                }
            }
            _ => (),
        },
        Event::MainEventsCleared => {
            if last_cpu.elapsed().as_millis() > 5 {
                let micro_time = last_cpu.elapsed().as_micros();
                let mut spent_time: u128 = 0;
                let mut executions_per_run = 0;
                while spent_time < micro_time {
                    executions_per_run = executions_per_run + 1;
                    if keep_trying {
                        let result = cpu.run_cycle(&mut mem, &mut state, &is_key_pressed);
                        match result {
                            Err(_) => {
                                keep_trying = false;
                                println!("{:?}", cpu.v)
                            }
                            _ => (),
                        }
                    }
                    spent_time = spent_time + one_cycle_time;
                }
                last_cpu = Instant::now();
            }
            if last_draw.elapsed().as_millis() > 16 && cpu.drawn {
                window.request_redraw();
                last_draw = Instant::now();
                if cpu.dt > 0 {
                    cpu.dt = cpu.dt - 1
                };
                if cpu.st > 0 {
                    sound_system.play();
                    cpu.st = cpu.st - 1
                } else {
                    sound_system.pause();
                }
            }

            *control_flow = ControlFlow::Poll
        }
        Event::RedrawRequested(_window_id) => {
            // Draw it to the `SurfaceTexture`
            let frame = pixels.get_frame();
            let chunks = frame.chunks_exact_mut(4);
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
        }
        Event::RedrawEventsCleared => {}
        _ => (),
    });
}

fn load_from_file(file: &str) -> Vec<u8> {
    return fs::read(file).expect("Failed to read the input file");
}
fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}
