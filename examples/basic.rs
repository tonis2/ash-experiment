use vulkan::{Swapchain, VkInstance};

use ash::version::DeviceV1_0;

use winit::event_loop::EventLoop;
fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("test")
        .with_inner_size(winit::dpi::LogicalSize::new(1500.0, 800.0))
        .build(&event_loop)
        .expect("Failed to create window.");

    let vulkan_base = VkInstance::new(&window);
    let swapchain = Swapchain::new(&vulkan_base, 1500, 800);
    let command_pool = vulkan_base.create_command_pool();

    unsafe {
        vulkan_base.device.destroy_command_pool(command_pool, None);
    }
}
