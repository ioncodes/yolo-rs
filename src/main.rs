#[macro_use]
extern crate glium;
extern crate clap;
extern crate flate2;

mod window;

use glium::{glutin, Surface};
use clap::{App, Arg};
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::prelude::*;
use std::thread;
use std::io;
use std::sync::mpsc::{self};

const DEFAULT_VERTEX: &'static str = include_str!("shaders/default.vert");
const DEFAULT_WIDTH: u32 = 1024;
const DEFAULT_HEIGHT: u32 = 786;

fn main() {
    let config = config();

    let fragment_shader = config.0;
    let mut vertex_shader = DEFAULT_VERTEX.to_string();
    if config.1 != "" {
        vertex_shader = read_shader(config.1);
    }
    let resolution = [config.2, config.3];
    let vsync = config.4;
    let add_time = config.5;
    let interactive = config.6;
    let debug = config.7;

    let (tx, rx) = mpsc::channel();

    let gl_thread = thread::spawn(move || setup_window(resolution, add_time, vsync, vertex_shader, fragment_shader, rx, debug));

    if interactive {
        loop {
            let input = io::stdin();
            let mut locked_input = input.lock();
            print!("$ ");
            let _ = io::stdout().flush();
            let mut command = String::new();
            locked_input.read_line(&mut command)
                        .expect("failed to read from stdin");
            command = command.trim_right_matches("\r\n").to_owned();
            if debug { println!("Command: {:?}", command); }
            if command == "pause" {
                let _ = tx.send(0);
            } else if command == "resume" {
                let _ = tx.send(1);
            } else if command == "exit" {
                let _ = tx.send(2);
                break;
            }
        }
    }

    let _ = gl_thread.join();
}

fn setup_window(resolution: [u32; 2], add_time: f32, vsync: bool, vertex_shader: String, fragment_shader: String, rx: mpsc::Receiver<i32>, debug: bool) {
    let mut window = window::Window::new(resolution, "yolo".to_owned(), vsync);
    let display = window.build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }

    implement_vertex!(Vertex, position);

    let shape = glium::vertex::VertexBuffer::new(&display, &[
        Vertex { position: [-1.0,  1.0] },
        Vertex { position: [ 1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] },
        Vertex { position: [ 1.0, -1.0] },
    ]).unwrap();

    let program =
    glium::Program::from_source(&display, &vertex_shader, &fragment_shader, None)
        .unwrap();

    let mut closed = false;
    let mut paused = false;

    // Uniforms
    let mut time: f32 = 0.0;
    let mut mouse = [0.0 as f32, 0.0 as f32];

    while !closed {
        if let Ok(r) = rx.try_recv() { 
            match r {
                0 => {
                    paused = true;
                    if debug { println!("Paused") };
                },
                1 => {
                    paused = false;
                    if debug { println!("Resumed") };
                },
                2 => {
                    closed = true;
                    if debug { println!("Exiting...") };
                },
                _ => {
                    if debug { println!("Unknown mode...") };
                }
            }
        }

        if !paused {
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 1.0, 1.0);
            let uniforms =
                uniform! { 
                    resolution: [resolution[0] as f32, resolution[1] as f32],
                    time: time,
                    mouse: mouse,
                };

            time += add_time;

            target
                .draw(
                    &shape,
                    &glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip),
                    &program,
                    &uniforms,
                    &Default::default(),
                )
                .unwrap();
            target.finish().unwrap();

            window.events_loop.poll_events(|event| {
                match event {
                    glutin::Event::WindowEvent{ event, .. } => match event {
                        glutin::WindowEvent::Closed => closed = true,
                        glutin::WindowEvent::MouseMoved { position, .. } => mouse = [position.0 as f32 / resolution[0] as f32, 1.0 - position.1 as f32 / resolution[1] as f32],
                        _ => ()
                    },
                    _ => ()
                }
            });
        }
    }
}

fn config() -> (String, String, u32, u32, bool, f32, bool, bool) {
    let matches = App::new("yolo")
        .args(&[
            Arg::with_name("vert")
                    .help("the vertex shader to load")
                    .takes_value(true)
                    .short("v")
                    .long("vert"),
            Arg::with_name("width")
                    .help("window width")
                    .takes_value(true)
                    .short("w")
                    .long("width"),
            Arg::with_name("height")
                    .help("window height")
                    .takes_value(true)
                    .short("h")
                    .long("height"),
            Arg::with_name("time")
                    .help("time speed (default 0.01)")
                    .takes_value(true)
                    .short("t")
                    .long("time"),
            Arg::with_name("vsync")
                    .help("enable vsync?")
                    .short("s")
                    .long("vsync"),
            Arg::with_name("decompress")
                    .help("decompress frag?")
                    .short("d")
                    .long("decompress"),
            Arg::with_name("interactive")
                    .help("start in interactive mode?")
                    .short("i")
                    .long("interactive"),
            Arg::with_name("debug")
                    .help("start in debug mode?")
                    .short("b")
                    .long("debug"),
            Arg::with_name("frag")
                    .help("the fragment shader to load")
                    .index(1)
                    .required(true)
        ])
        .get_matches();

    let mut frag = matches.value_of("frag").unwrap().to_string();
    let frag_name = matches.value_of("frag").unwrap().to_string();
    println!("Loading fragment shader: {}", frag);

    if matches.is_present("decompress") {
        let mut file = File::open(&frag).unwrap();
        let mut contents = Vec::new();
        let _ = file.read_to_end(&mut contents).unwrap();
        let mut d = GzDecoder::new(&contents[..]).unwrap();
        let mut s = String::new();
        d.read_to_string(&mut s).unwrap();
        frag = s;
    } else {
        frag = read_shader(frag);
    }

    let mut vertex: String = "".to_owned();
    if let Some(vert) = matches.value_of("vert") {
        println!("Loading custom vertex shader: {}", vert);
        vertex = vert.to_string();
    }

    let mut screen_width: u32 = DEFAULT_WIDTH;
    if let Some(width) = matches.value_of("width") {
        println!("Setting width to: {}", width);
        screen_width = width.to_string().parse::<u32>().unwrap();
    }

    let mut screen_height: u32 = DEFAULT_HEIGHT;
    if let Some(height) = matches.value_of("height") {
        println!("Setting height to: {}", height);
        screen_height = height.to_string().parse::<u32>().unwrap();
    }

    let mut config_vsync = false;
    if matches.is_present("vsync") {
        println!("vsync enabled!");
        config_vsync = true;
    }

    let mut config_time = 0.01;
    if let Some(time) = matches.value_of("time") {
        println!("Time: {}", time);
        config_time = time.to_string().parse::<f32>().unwrap();
    }

    let mut config_interactive = false;
    if matches.is_present("interactive") {
        println!("interactive enabled!");
        config_interactive = true;
    }

    let mut config_debug = false;
    if matches.is_present("debug") {
        println!("debug mode enabled!");
        config_debug = true;
    }

    println!("\n");

    println!("=======================================");
    println!("Fragment Shader: {:?}", frag_name);
    println!("Vertex Shader:   {:?}", vertex);
    println!("Screen Width:    {:?}", screen_width);
    println!("Screen Height:   {:?}", screen_height);
    println!("VSync:           {:?}", config_vsync);
    println!("Time:            {:?}", config_time);
    println!("Interactive:     {:?}", config_interactive);
    println!("Debug:           {:?}", config_debug);
    println!("=======================================");

    println!("\n");

    println!("All tests passed! And keep in mind:");

    println!("\n");

    println!(r"  _        _          _            _             _     ");
    println!(r"/\ \     /\_\       /\ \         _\ \          /\ \    ");
    println!(r"\ \ \   / / /      /  \ \       /\__ \        /  \ \   ");
    println!(r" \ \ \_/ / /      / /\ \ \     / /_ \_\      / /\ \ \  ");
    println!(r"  \ \___/ /      / / /\ \ \   / / /\/_/     / / /\ \ \ ");
    println!(r"   \ \ \_/      / / /  \ \_\ / / /         / / /  \ \_\");
    println!(r"    \ \ \      / / /   / / // / /         / / /   / / /");
    println!(r"     \ \ \    / / /   / / // / / ____    / / /   / / / ");
    println!(r"      \ \ \  / / /___/ / // /_/_/ ___/\ / / /___/ / /  ");
    println!(r"       \ \_\/ / /____\/ //_______/\__\// / /____\/ /   ");
    println!(r"        \/_/\/_________/ \_______\/    \/_________/    ");

    println!("\n");

    (frag, vertex, screen_width, screen_height, config_vsync, config_time, config_interactive, config_debug)
}

fn read_shader(file: String) -> String {
    let mut file = File::open(file).unwrap();
    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents).unwrap();
    contents
}
