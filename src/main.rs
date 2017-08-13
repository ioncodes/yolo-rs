#[macro_use]
extern crate glium;
extern crate clap;
extern crate flate2;
extern crate notify;

mod window;

use glium::{glutin, Surface, Api, Profile, Version};
use clap::{App, Arg};
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::prelude::*;
use std::thread;
use std::io;
use std::sync::mpsc::{self};
use notify::{RecommendedWatcher, Watcher, RecursiveMode};
use std::time::Duration;

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
    let reload = config.8;
    let frag_name = config.9;
    let msaa = config.10;
    let borderless = config.11;

    let (tx, rx) = mpsc::channel();
    let (shader_sender, shader_receiver) = mpsc::channel();

    let gl_thread = thread::spawn(move || setup_window(resolution, add_time, vsync, vertex_shader, fragment_shader, rx, shader_receiver, debug, msaa, borderless));

    if interactive {
        let _ = thread::spawn(move || run_interactive(debug, tx));
    }

    if reload {
        let _ = thread::spawn(move || run_watcher(frag_name, shader_sender));
    }

    let _ = gl_thread.join();
}

fn show_commands() {
    println!("pause");
    println!("resume");
    println!("exit");
}

fn run_interactive(debug: bool, tx: mpsc::Sender<i32>) {
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
        } else if command == "help" {
            show_commands();
        } else {
            println!("Unknown command");
            show_commands();
        }
    }
}

fn run_watcher(file: String, shader_sender: mpsc::Sender<String>) {
    let (tx, rx) = mpsc::channel();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2)).unwrap();

    watcher.watch(&file, RecursiveMode::NonRecursive).unwrap();

    loop {
        match rx.recv() {
            Ok(_) => {
                std::thread::sleep(std::time::Duration::from_millis(100));
                let source = read_shader(file.to_owned());
                let _ = shader_sender.send(source);
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

fn setup_window(resolution: [u32; 2], add_time: f32, vsync: bool, vertex_shader: String, fragment_shader: String, rx: mpsc::Receiver<i32>, shader_receiver: mpsc::Receiver<String>, debug: bool, msaa: u16, borderless: bool) {
    let mut window = window::Window::new(resolution, "yolo".to_owned(), vsync, msaa, borderless);
    let display = window.build_display();

    if debug {
        print_opengl_info(&display);
    }

    let mut frag = fragment_shader;

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

    let mut program =
        glium::Program::from_source(&display, &vertex_shader, &frag, None)
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
        if let Ok(f) = shader_receiver.try_recv() { 
            frag = f;
            program =
                glium::Program::from_source(&display, &vertex_shader, &frag, None)
                .unwrap();
            time = 0.0;
            mouse = [0.0 as f32, 0.0 as f32];
            println!("Reloaded!");
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

fn config() -> (String, String, u32, u32, bool, f32, bool, bool, bool, String, u16, bool) {
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
            Arg::with_name("msaa")
                    .help("MSAA level [2,4,8,16]")
                    .takes_value(true)
                    .short("m")
                    .long("msaa"),
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
            Arg::with_name("reload")
                    .help("reload on file changes?")
                    .short("r")
                    .long("reload"),
            Arg::with_name("borderless")
                    .help("Open borderless?")
                    .short("l")
                    .long("borderless"),
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

    let mut config_msaa: u16 = 0;
    if let Some(msaa) = matches.value_of("msaa") {
        println!("Setting MSAA to: {}", msaa);
        config_msaa = msaa.to_string().parse::<u16>().unwrap();
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

    let mut config_reload = false;
    if matches.is_present("reload") {
        println!("reload mode enabled!");
        config_reload = true;
    }

    let mut config_borderless = false;
    if matches.is_present("borderless") {
        println!("borderless mode enabled!");
        config_borderless = true;
    }

    println!("\n");

    println!("=======================================");
    println!("Fragment Shader: {:?}", frag_name);
    println!("Vertex Shader:   {:?}", vertex);
    println!("Screen Width:    {:?}", screen_width);
    println!("Screen Height:   {:?}", screen_height);
    println!("MSAA:            {:?}x", config_msaa);
    println!("VSync:           {:?}", config_vsync);
    println!("Time:            {:?}", config_time);
    println!("Interactive:     {:?}", config_interactive);
    println!("Debug:           {:?}", config_debug);
    println!("Reload:          {:?}", config_reload);
    println!("Borderless:      {:?}", config_borderless);
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

    (frag, vertex, screen_width, screen_height, config_vsync, config_time, config_interactive, config_debug, config_reload, frag_name, config_msaa, config_borderless)
}

fn read_shader(file: String) -> String {
    let mut file = File::open(file).unwrap();
    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents).unwrap();
    contents
}

fn print_opengl_info(display: &glium::Display) {
    let version = *display.get_opengl_version();
    let api = match version {
        Version(Api::Gl, _, _) => "OpenGL",
        Version(Api::GlEs, _, _) => "OpenGL ES"
    };

    println!("{} context verson: {}", api, display.get_opengl_version_string());

    print!("{} context flags:", api);
    if display.is_forward_compatible() {
        print!(" forward-compatible");
    }
    if display.is_debug() {
        print!(" debug");
    }
    if display.is_robust() {
        print!(" robustness");
    }
    print!("\n");

    if version >= Version(Api::Gl, 3, 2) {
        println!("{} profile mask: {}", api,
                 match display.get_opengl_profile() {
                     Some(Profile::Core) => "core",
                     Some(Profile::Compatibility) => "compatibility",
                     None => "unknown"
                 });
    }

    println!("{} robustness strategy: {}", api,
             if display.is_context_loss_possible() {
                 "lose"
             } else {
                 "none"
             });
    
    println!("{} context renderer: {}", api, display.get_opengl_renderer_string());
    println!("{} context vendor: {}", api, display.get_opengl_vendor_string());
}