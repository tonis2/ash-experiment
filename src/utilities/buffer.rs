use ash::vk;
use crate::VkInstance;
use ash::version::DeviceV1_0;

pub struct Buffer {
    pub size: u32,
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub memory_requirements: vk::MemoryRequirements,
}

impl Buffer {
    pub fn destroy(&mut self, vulkan: &VkInstance) {
        unsafe {
            vulkan.device.destroy_buffer(self.buffer, None);
            vulkan.device.free_memory(self.memory, None);
        }
    }
}