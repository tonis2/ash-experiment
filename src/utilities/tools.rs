use crate::Context;
use ash::util::read_spv;
use ash::version::DeviceV1_0;
use ash::vk;
use std::{
    default::Default,
    ffi::{CStr, CString},
    os::raw::c_char,
    path::Path,
    sync::Arc,
};

pub struct Shader {
    pub shader_info: vk::PipelineShaderStageCreateInfo,
    shader_module: vk::ShaderModule,
    context: Arc<Context>,
}

impl Shader {
    pub fn new(
        path: &Path,
        stage: vk::ShaderStageFlags,
        entry_name: &CString,
        context: Arc<Context>,
    ) -> Self {
        let shader_module = unsafe {
            context
                .device
                .create_shader_module(
                    &vk::ShaderModuleCreateInfo::builder().code(&load_shader(path)),
                    None,
                )
                .expect("Vertex shader module error")
        };

        Self {
            shader_info: vk::PipelineShaderStageCreateInfo {
                module: shader_module,
                p_name: entry_name.as_ptr(),
                stage: stage,
                ..Default::default()
            },
            shader_module,
            context: context,
        }
    }
    pub fn info(&self) -> vk::PipelineShaderStageCreateInfo {
        self.shader_info
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.context
                .device
                .destroy_shader_module(self.shader_module, None);
        }
    }
}

/// Helper function to convert [c_char; SIZE] to string
pub fn vk_to_string(raw_string_array: &[c_char]) -> String {
    let raw_string = unsafe {
        let pointer = raw_string_array.as_ptr();
        CStr::from_ptr(pointer)
    };

    raw_string
        .to_str()
        .expect("Failed to convert vulkan raw string.")
        .to_owned()
}

pub fn load_shader(shader_path: &Path) -> Vec<u32> {
    use std::fs::File;
    use std::io::Read;
    let mut shader_data =
        File::open(shader_path).expect(&format!("failed to read shader {:?}", shader_path));
    let mut buffer = Vec::new();
    shader_data
        .read_to_end(&mut buffer)
        .expect("Failed to load shader data");

    read_spv(&mut shader_data).expect("Failed to read vertex shader spv file")
}

pub fn find_memorytype_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    // Try to find an exactly matching memory flag
    let best_suitable_index =
        find_memorytype_index_f(memory_req, memory_prop, flags, |property_flags, flags| {
            property_flags == flags
        });
    if best_suitable_index.is_some() {
        return best_suitable_index;
    }
    // Otherwise find a memory flag that works
    find_memorytype_index_f(memory_req, memory_prop, flags, |property_flags, flags| {
        property_flags & flags == flags
    })
}

pub fn find_memorytype_index_f<F: Fn(vk::MemoryPropertyFlags, vk::MemoryPropertyFlags) -> bool>(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
    f: F,
) -> Option<u32> {
    let mut memory_type_bits = memory_req.memory_type_bits;
    for (index, ref memory_type) in memory_prop.memory_types.iter().enumerate() {
        if memory_type_bits & 1 == 1 && f(memory_type.property_flags, flags) {
            return Some(index as u32);
        }
        memory_type_bits >>= 1;
    }
    None
}

// Simple offset_of macro akin to C++ offsetof
#[macro_export]
macro_rules! offset_of {
    ($base:path, $field:ident) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let b: $base = mem::zeroed();
            (&b.$field as *const _ as isize) - (&b as *const _ as isize)
        }
    }};
}

pub fn find_memory_type(
    type_filter: u32,
    required_properties: vk::MemoryPropertyFlags,
    mem_properties: &vk::PhysicalDeviceMemoryProperties,
) -> u32 {
    for (i, memory_type) in mem_properties.memory_types.iter().enumerate() {
        if (type_filter & (1 << i)) > 0 && memory_type.property_flags.contains(required_properties)
        {
            return i as u32;
        }
    }

    panic!("Failed to find suitable memory type!")
}

/// This method will convert any slice to a byte slice.
/// Use with slices of number primitives.
pub unsafe fn as_byte_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
}

pub trait MeshTrait<T> {
    fn get_indices(&mut self) -> Vec<u32>;
    fn get_vertices(&mut self) -> Vec<T>;
}

#[derive(Debug, Clone)]
pub struct Mesh<T: Clone> {
    pub vertices: Vec<T>,
    pub indices: Vec<u32>,
}

impl<T: Clone> Default for Mesh<T> {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl<T: Clone> MeshTrait<T> for Mesh<T> {
    fn get_indices(&mut self) -> Vec<u32> {
        self.indices.clone()
    }
    fn get_vertices(&mut self) -> Vec<T> {
        self.vertices.clone()
    }
}

#[derive(Clone)]
pub struct Batch<T: Clone> {
    pub indices: Vec<u32>,
    pub vertices: Vec<T>,
}

impl<T: Clone> Batch<T> {
    pub fn new() -> Self {
        Self {
            indices: Vec::new(),
            vertices: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn add(&mut self, mesh: &mut Mesh<T>) {
        for indice in mesh.get_indices() {
            self.indices
                .push((indice as i64 + self.vertices.len() as i64) as u32);
        }

        self.vertices.extend(mesh.get_vertices());
    }
}
