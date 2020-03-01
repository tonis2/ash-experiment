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
    unsafe {
        let indices_slice = &indices[..];
        let index_input_buffer_info = vk::BufferCreateInfo {
            size: std::mem::size_of_val(&indices_slice) as u64,
            usage: vk::BufferUsageFlags::INDEX_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let mut buffer = vulkan.create_buffer(index_input_buffer_info);

        let index_ptr = vulkan
            .device
            .map_memory(
                buffer.memory,
                0,
                buffer.memory_requirements.size,
                vk::MemoryMapFlags::empty(),
            )
            .unwrap();
        let mut index_slice = Align::new(
            index_ptr,
            align_of::<u32>() as u64,
            buffer.memory_requirements.size,
        );

        index_slice.copy_from_slice(&indices);

        vulkan.device.unmap_memory(buffer.memory);
        vulkan.device
            .bind_buffer_memory(buffer.buffer, buffer.memory, 0)
            .unwrap();
        buffer.size = indices.len() as u32;
        buffer
    }
}

pub fn create_vertex_buffer<A: Copy>(
    vertices: &[A],
    base: &VkInstance,
    vertex: &VertexDescriptor,
) -> Buffer {
    unsafe {
        let vertex_input_buffer_info = vk::BufferCreateInfo {
            size: vertex.size,
            usage: vk::BufferUsageFlags::VERTEX_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let mut buffer = base.create_buffer(vertex_input_buffer_info);
        let vert_ptr = base
            .device
            .map_memory(
                buffer.memory,
                0,
                buffer.memory_requirements.size,
                vk::MemoryMapFlags::empty(),
            )
            .unwrap();

        let mut vert_align = Align::new(vert_ptr, vertex.align, buffer.memory_requirements.size);

        vert_align.copy_from_slice(&vertices);

        base.device.unmap_memory(buffer.memory);
        base.device
            .bind_buffer_memory(buffer.buffer, buffer.memory, 0)
            .unwrap();
        buffer.size = vertices.len() as u32;
        buffer
    }
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