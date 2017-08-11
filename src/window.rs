use glium::{Display};
use glium::glutin::{WindowBuilder, ContextBuilder, EventsLoop};

pub struct Window {
	pub resolution: [u32; 2],
	pub title: String,
	pub vsync: bool,
	pub events_loop: EventsLoop
}

impl Window {
	pub fn new(resolution: [u32; 2], title: String, vsync: bool) -> Window {
		Window {
			resolution: resolution,
			title: title,
			vsync: vsync,
			events_loop: EventsLoop::new()
		}
	}

	pub fn build_display(&mut self) -> Display {
		let window = WindowBuilder::new()
                        .with_dimensions(1024, 768)
                        .with_title("yolo");
		let context = ContextBuilder::new()
        				.with_vsync(true);
		Display::new(window, context, &self.events_loop).unwrap()
	}
}