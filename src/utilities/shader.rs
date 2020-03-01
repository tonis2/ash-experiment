use std::default::Default;
use std::fs::File;
use std::io::Read;
use std::mem::align_of;

use ash::util::*;
use ash::vk;

use crate::utilities::buffer::Buffer;
use crate::VkInstance;
use ash::version::DeviceV1_0;

pub struct VertexDescriptor {
    pub binding_descriptor: Vec<vk::VertexInputBindingDescription>,
    pub attribute_descriptor: Vec<vk::VertexInputAttributeDescription>,
    pub size: u64,
    pub align: u64,
}

pub fn create_index_buffer(indices: &Vec<u16>, vulkan: &VkInstance) -> Buffer {
    let indices_slice = &indices[..];
    let index_input_buffer_info = vk::BufferCreateInfo {
        size: std::mem::size_of_val(&indices_slice) as u64,
        usage: vk::BufferUsageFlags::INDEX_BUFFER,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };
    let mut buffer = vulkan.create_buffer(index_input_buffer_info);

    buffer.copy_to_buffer(align_of::<u32>() as u64, &indices, &vulkan);
    buffer
}

pub fn create_vertex_buffer<A: Copy>(
    vertices: &[A],
    base: &VkInstance,
    vertex: &VertexDescriptor,
    vulkan: &VkInstance,
) -> Buffer {
    let vertex_input_buffer_info = vk::BufferCreateInfo {
        size: vertex.size,
        usage: vk::BufferUsageFlags::VERTEX_BUFFER,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };
    let mut buffer = base.create_buffer(vertex_input_buffer_info);
    buffer.copy_to_buffer(vertex.align, &vertices, &vulkan);
    buffer
}

pub struct Shader {
    pub vertex_shader: &'static str,
    pub fragment_shader: &'static str,
}

impl Shader {
    pub fn load(&mut self, base: &VkInstance) -> (vk::ShaderModule, vk::ShaderModule) {
        unsafe {
            let mut vertex_shader = File::open(self.vertex_shader).expect("failed to read font");
            let mut buffer = Vec::new();
            vertex_shader
                .read_to_end(&mut buffer)
                .expect("failed to read file");

            let mut fragment_shader =
                File::open(self.fragment_shader).expect("failed to read font");
            let mut buffer = Vec::new();
            fragment_shader
                .read_to_end(&mut buffer)
                .expect("failed to read file");

            let vertex_code =
                read_spv(&mut vertex_shader).expect("Failed to read vertex shader spv file");
            let vertex_shader_info = vk::ShaderModuleCreateInfo::builder().code(&vertex_code);

            let frag_code =
                read_spv(&mut fragment_shader).expect("Failed to read fragment shader spv file");
            let frag_shader_info = vk::ShaderModuleCreateInfo::builder().code(&frag_code);

            let vertex_shader_module = base
                .device
                .create_shader_module(&vertex_shader_info, None)
                .expect("Vertex shader module error");

            let fragment_shader_module = base
                .device
                .create_shader_module(&frag_shader_info, None)
                .expect("Fragment shader module error");

            (vertex_shader_module, fragment_shader_module)
        }
    }
}
