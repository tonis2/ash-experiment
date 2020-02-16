mod pipelines;
use ash::{version::DeviceV1_0, vk};
use std::mem::{self, align_of};
use vulkan::{offset_of, Swapchain, VertexDescriptor, VkInstance};
use winit::event_loop::EventLoop;

#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub color: [f32; 4],
}
fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("test")
        .with_inner_size(winit::dpi::LogicalSize::new(1500.0, 800.0))
        .build(&event_loop)
        .expect("Failed to create window.");

    let vulkan_base = VkInstance::new(&window);
    let command_pool = vulkan_base.create_command_pool();

    let swapchain = Swapchain::new(&vulkan_base, 1500, 800);

    let render_pass = swapchain.create_render_pass();

    let vertex_descriptor = VertexDescriptor {
        binding_len: 1,
        descriptor_len: 2,
        binding_descriptor: vec![vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }],
        attribute_descriptor: vec![
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, color) as u32,
            },
        ],
        size: 3 * std::mem::size_of::<Vertex>() as u64,
        align: align_of::<Vertex>() as u64,
    };

    let (pipeline, layout) =
        pipelines::default::create_pipeline(&swapchain, render_pass, &vertex_descriptor);

    {
        let command_buffers = vulkan_base.create_command_buffers(command_pool, 2);
    }
    unsafe {
        vulkan_base.device.destroy_command_pool(command_pool, None);
        vulkan_base.device.destroy_render_pass(render_pass, None);
        // vulkan_base.device.destroy_pipeline(pipeline[0], None);
    }
}
