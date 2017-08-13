use glium::Display;
use glium::glutin::{WindowBuilder, ContextBuilder, EventsLoop};

pub struct Window {
    pub resolution: [u32; 2],
    pub title: String,
    pub vsync: bool,
    pub events_loop: EventsLoop,
    pub msaa: u16,
    pub borderless: bool,
}

impl Window {
    pub fn new(
        resolution: [u32; 2],
        title: String,
        vsync: bool,
        msaa: u16,
        borderless: bool,
    ) -> Window {
        Window {
            resolution: resolution,
            title: title,
            vsync: vsync,
            events_loop: EventsLoop::new(),
            msaa: msaa,
            borderless: borderless,
        }
    }

    pub fn build_display(&mut self) -> Display {
        let window = WindowBuilder::new()
            .with_dimensions(self.resolution[0], self.resolution[1])
            .with_title("yolo")
            .with_decorations(!self.borderless);
        let context = ContextBuilder::new()
            .with_vsync(self.vsync)
            .with_multisampling(self.msaa);
        Display::new(window, context, &self.events_loop).unwrap()
    }
}
