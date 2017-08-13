use glium::Display;
use glium::glutin::{WindowBuilder, ContextBuilder, EventsLoop};
use glium;

pub struct Window {
    pub resolution: [u32; 2],
    pub title: String,
    pub vsync: bool,
    pub events_loop: EventsLoop,
    pub msaa: u16,
    pub borderless: bool,
    pub fullscreen: bool,
}

impl Window {
    pub fn new(
        resolution: [u32; 2],
        title: String,
        vsync: bool,
        msaa: u16,
        borderless: bool,
        fullscreen: bool,
    ) -> Window {
        Window {
            resolution: resolution,
            title: title,
            vsync: vsync,
            events_loop: EventsLoop::new(),
            msaa: msaa,
            borderless: borderless,
            fullscreen: fullscreen,
        }
    }

    pub fn build_display(&mut self) -> Display {
        let window: WindowBuilder;
        if self.fullscreen {
            window = WindowBuilder::new().with_title("yolo").with_fullscreen(
                glium::glutin::get_available_monitors().nth(0).unwrap(),
            );
        } else {
            window = WindowBuilder::new()
                .with_dimensions(self.resolution[0], self.resolution[1])
                .with_title("yolo")
                .with_decorations(!self.borderless);
        }

        let context = ContextBuilder::new()
            .with_vsync(self.vsync)
            .with_multisampling(self.msaa);
        Display::new(window, context, &self.events_loop).unwrap()
    }
}
