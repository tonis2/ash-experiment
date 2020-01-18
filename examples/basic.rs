use wrapper::{app, Base};
struct Main;

impl Base for Main {
   fn init() -> (Self, winit::window::WindowBuilder) {
      const WINDOW_TITLE: &'static str = "Vulkan app";
      const WINDOW_WIDTH: u32 = 800;
      const WINDOW_HEIGHT: u32 = 600;

      let window = winit::window::WindowBuilder::new()
         .with_title(WINDOW_TITLE)
         .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));

      (Main {}, window)
   }

   fn update() {}

   fn render() {}
}

fn main() {
   app::<Main>()
}
