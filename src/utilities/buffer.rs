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
    pub fn copy_to_buffer<D: Copy>(&mut self, align: u64, data: &[D], vulkan: &VkInstance) {
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
            vulkan
                .device
                .bind_buffer_memory(self.buffer, self.memory, 0)
                .unwrap();
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
