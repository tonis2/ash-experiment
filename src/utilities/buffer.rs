use crate::VkInstance;
use ash::version::DeviceV1_0;
use ash::vk;

use ash::util::Align;

pub struct Buffer {
    pub size: u32,
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub memory_requirements: vk::MemoryRequirements,
}

impl Buffer {
    //Copy to buffer when size can be dynamic
    pub fn copy_to_buffer_dynamic<D: Copy>(&mut self, align: u64, data: &[D], vulkan: &VkInstance) {
        unsafe {
            let index_ptr = vulkan
                .device
                .map_memory(
                    self.memory,
                    0,
                    self.memory_requirements.size,
                    vk::MemoryMapFlags::empty(),
                )
                .unwrap();
            let mut data_slice = Align::new(index_ptr, align, self.memory_requirements.size);

            data_slice.copy_from_slice(data);

            vulkan.device.unmap_memory(self.memory);

            self.size = data.len() as u32;
        }
    }

    //copy to buffer with fixed size
    pub fn copy_buffer<T>(&mut self, size: u64, data: &Vec<T>, vulkan: &VkInstance) {
        unsafe {
            let data_ptr = vulkan
                .device
                .map_memory(self.memory, 0, size, vk::MemoryMapFlags::empty())
                .expect("Failed to Map Memory") as *mut T;

            data_ptr.copy_from_nonoverlapping(data[..].as_ptr(), data.len());

            vulkan.device.unmap_memory(self.memory);

            self.size = data.len() as u32;
        }
    }

    pub fn destroy(&mut self, vulkan: &VkInstance) {
        unsafe {
            vulkan.device.destroy_buffer(self.buffer, None);
            vulkan.device.free_memory(self.memory, None);
        }
    }
}
