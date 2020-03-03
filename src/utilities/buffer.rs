use crate::VkInstance;
use ash::version::DeviceV1_0;
use ash::vk;

use ash::util::Align;

pub struct Buffer {
    pub size: u32,
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub usage: vk::BufferUsageFlags,
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
    pub fn copy_buffer<T>(&mut self, data: &Vec<T>, vulkan: &VkInstance) {
        let buffer_size = ::std::mem::size_of_val(data) as vk::DeviceSize;
        unsafe {
            let data_ptr = vulkan
                .device
                .map_memory(self.memory, 0, buffer_size, vk::MemoryMapFlags::empty())
                .expect("Failed to Map Memory") as *mut T;

            data_ptr.copy_from_nonoverlapping(data[..].as_ptr(), data.len());

            vulkan.device.unmap_memory(self.memory);

            self.size = data.len() as u32;
        }
    }

    // pub fn copy_buffer_2<T>(&mut self, data: &Vec<T>, vulkan: &VkInstance) {
    //     let buffer_size = ::std::mem::size_of_val(data) as vk::DeviceSize;
    //     let staging_buffer_create_info = vk::BufferCreateInfo {
    //         size: buffer_size,
    //         usage: vk::BufferUsageFlags::TRANSFER_SRC,
    //         sharing_mode: vk::SharingMode::EXCLUSIVE,
    //         ..Default::default()
    //     };
    //     let staging_buffer = vulkan.create_buffer(staging_buffer_create_info);

    //     unsafe {
    //         let data_ptr = vulkan
    //             .device
    //             .map_memory(
    //                 staging_buffer.memory,
    //                 0,
    //                 buffer_size,
    //                 vk::MemoryMapFlags::empty(),
    //             )
    //             .expect("Failed to Map Memory") as *mut T;

    //         data_ptr.copy_from_nonoverlapping(data[..].as_ptr(), data.len());

    //         vulkan.device.unmap_memory(staging_buffer.memory);
    //     }

    //     let buffer_create_info = vk::BufferCreateInfo {
    //         size: buffer_size,
    //         usage: self.usage,
    //         sharing_mode: vk::SharingMode::EXCLUSIVE,
    //         ..Default::default()
    //     };

    //     let buffer = vulkan.create_buffer(buffer_create_info);

    //     vulkan.copy_buffer(staging_buffer.buffer, buffer.buffer, buffer_size);
    // }

    pub fn destroy(&mut self, vulkan: &VkInstance) {
        unsafe {
            vulkan.device.destroy_buffer(self.buffer, None);
            vulkan.device.free_memory(self.memory, None);
        }
    }
}
