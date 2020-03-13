use crate::VkInstance;
use ash::version::DeviceV1_0;
use ash::vk;

use ash::util::Align;
use std::rc::Rc;

#[derive(Clone)]
pub struct Buffer {
    pub size: u32,
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub usage: vk::BufferUsageFlags,
    pub memory_requirements: vk::MemoryRequirements,
    pub device: Rc<ash::Device>,
}

impl Buffer {
    //Copy to buffer when size can be dynamic
    pub fn copy_to_buffer_dynamic<D: Copy>(&mut self, align: u64, data: &[D]) {
        unsafe {
            let index_ptr = self
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

            self.device.unmap_memory(self.memory);
        }
    }

    //copy to buffer with fixed size
    pub fn copy_buffer<T>(&mut self, data: &Vec<T>, vulkan: &VkInstance) {
        unsafe {
            let data_ptr = vulkan
                .device
                .map_memory(self.memory, 0, self.size as u64, vk::MemoryMapFlags::empty())
                .expect("Failed to Map Memory") as *mut T;

            data_ptr.copy_from_nonoverlapping(data[..].as_ptr(), data.len());

            vulkan.device.unmap_memory(self.memory);
        }
    }

    pub fn destroy(&self) {
        unsafe {
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
    }
}

// impl Drop for Buffer {
//     fn drop(&mut self) {
//         unsafe {
//             self.device.destroy_buffer(self.buffer, None);
//             self.device.free_memory(self.memory, None);
//         }
//     }
// }
