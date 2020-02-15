use std::default::Default;
use std::fs::File;
use std::io::Read;
use std::mem::align_of;

use super::helpers::*;
use ash::util::*;
use ash::vk;

use ash::version::DeviceV1_0;
use crate::utility::buffer::Buffer;
use crate::VkInstance;

pub struct VertexDescriptor {
    pub binding_len: i32,
    pub descriptor_len: i32,
    pub binding_descriptor: Vec<vk::VertexInputBindingDescription>,
    pub attribute_descriptor: Vec<vk::VertexInputAttributeDescription>,
    pub size: u64,
    pub align: u64,
}

pub fn create_index_buffer(indices: &Vec<u16>, base: &VkInstance) -> Buffer {
    unsafe {
        let indices_slice = &indices[..];
        let index_buffer_info = vk::BufferCreateInfo::builder()
            .size(std::mem::size_of_val(&indices_slice) as u64)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let index_buffer = base.device.create_buffer(&index_buffer_info, None).unwrap();
        let index_buffer_memory_req = base.device.get_buffer_memory_requirements(index_buffer);
        let index_buffer_memory_index = find_memorytype_index(
            &index_buffer_memory_req,
            &base.get_physical_device_memory_properties(),
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
        .expect("Unable to find suitable memorytype for the index buffer.");

        let index_allocate_info = vk::MemoryAllocateInfo {
            allocation_size: index_buffer_memory_req.size,
            memory_type_index: index_buffer_memory_index,
            ..Default::default()
        };
        let index_buffer_memory = base
            .device
            .allocate_memory(&index_allocate_info, None)
            .unwrap();
        let index_ptr = base
            .device
            .map_memory(
                index_buffer_memory,
                0,
                index_buffer_memory_req.size,
                vk::MemoryMapFlags::empty(),
            )
            .unwrap();
        let mut index_slice = Align::new(
            index_ptr,
            align_of::<u32>() as u64,
            index_buffer_memory_req.size,
        );

        index_slice.copy_from_slice(&indices);

        base.device.unmap_memory(index_buffer_memory);
        base.device
            .bind_buffer_memory(index_buffer, index_buffer_memory, 0)
            .unwrap();

        Buffer {
            size: indices.len() as u32,
            buffer: index_buffer,
            memory: index_buffer_memory,
        }
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

        let vertex_input_buffer = base
            .device
            .create_buffer(&vertex_input_buffer_info, None)
            .unwrap();

        let vertex_input_buffer_memory_req = base
            .device
            .get_buffer_memory_requirements(vertex_input_buffer);

        let vertex_input_buffer_memory_index = find_memorytype_index(
            &vertex_input_buffer_memory_req,
            &base.get_physical_device_memory_properties(),
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
        .expect("Unable to find suitable memorytype for the vertex buffer.");

        let vertex_buffer_allocate_info = vk::MemoryAllocateInfo {
            allocation_size: vertex_input_buffer_memory_req.size,
            memory_type_index: vertex_input_buffer_memory_index,
            ..Default::default()
        };

        let vertex_input_buffer_memory = base
            .device
            .allocate_memory(&vertex_buffer_allocate_info, None)
            .unwrap();

        let vert_ptr = base
            .device
            .map_memory(
                vertex_input_buffer_memory,
                0,
                vertex_input_buffer_memory_req.size,
                vk::MemoryMapFlags::empty(),
            )
            .unwrap();

        let mut vert_align =
            Align::new(vert_ptr, vertex.align, vertex_input_buffer_memory_req.size);

        vert_align.copy_from_slice(&vertices);

        base.device.unmap_memory(vertex_input_buffer_memory);
        base.device
            .bind_buffer_memory(vertex_input_buffer, vertex_input_buffer_memory, 0)
            .unwrap();

        Buffer {
            size: vertices.len() as u32,
            buffer: vertex_input_buffer,
            memory: vertex_input_buffer_memory,
        }
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
