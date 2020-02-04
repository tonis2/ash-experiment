use vulkan;
use winit::event_loop::EventLoop;
fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
    .with_title("test")
    .with_inner_size(winit::dpi::LogicalSize::new(1500.0, 800.0))
    .build(&event_loop)
    .expect("Failed to create window.");

   let vulkan_base = vulkan::VulkanBase::new(&window);
}