use crate::Context;
use ash::vk;
use std::sync::Arc;

#[derive(Clone)]
pub struct Buffer {
    pub buffer: vk::Buffer,
    pub allocation: vk_mem::Allocation,
    pub allocation_info: vk_mem::AllocationInfo,
    pub size: vk::DeviceSize,
    pub context: Arc<Context>,
}

impl Buffer {
    pub fn new(
        allocation_create_info: &vk_mem::AllocationCreateInfo,
        buffer_create_info: &vk::BufferCreateInfo,
        context: Arc<Context>,
    ) -> Buffer {
        let (buffer, allocation, allocation_info) = context
            .memory
            .create_buffer(buffer_create_info, &allocation_create_info)
            .expect("Failed to create buffer!");

        Buffer {
            buffer,
            allocation,
            allocation_info,
            size: buffer_create_info.size,
            context,
        }
    }

    pub fn new_mapped_basic(
        size: vk::DeviceSize,
        buffer_usage: vk::BufferUsageFlags,
        memory_usage: vk_mem::MemoryUsage,
        context: Arc<Context>,
    ) -> Self {
        let allocation_create_info = vk_mem::AllocationCreateInfo {
            usage: memory_usage,
            ..Default::default()
        };

        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(buffer_usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();

        Buffer::new(&allocation_create_info, &buffer_create_info, context)
    }

    pub fn upload_to_buffer<T: Copy>(&self, data: &[T], offset: u64) {
        let alignment = std::mem::align_of::<T>() as _;
        let data_pointer = self.map_memory().expect("Failed to map memory!");
        unsafe {
            let mut align = ash::util::Align::new(
                data_pointer.add(offset as usize) as _,
                alignment,
                self.allocation_info.get_size() as _,
            );
            align.copy_from_slice(data);
        }
      
    }

    pub fn map_memory(&self) -> vk_mem::error::Result<*mut u8> {
        self.context.memory.map_memory(&self.allocation)
    }

    pub fn unmap_memory(&self) -> vk_mem::error::Result<()> {
        self.context.memory.unmap_memory(&self.allocation)
    }

    pub fn flush(&self, offset: usize, size: usize) -> vk_mem::error::Result<()> {
        self.context
            .memory
            .flush_allocation(&self.allocation, offset, size)
    }

    pub fn allocation_size(&self) -> u64 {
        self.allocation_info.get_size() as u64
    }

    pub fn offset(&self) -> u64 {
        self.allocation_info.get_offset() as u64
    }

    pub fn descriptor_info(&self, offset: u64) -> vk::DescriptorBufferInfo {
        vk::DescriptorBufferInfo {
            buffer: self.buffer,
            offset: offset,
            range: self.size,
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        self.context.wait_idle();
        self.context
            .memory
            .destroy_buffer(self.buffer, &self.allocation)
            .expect("Failed to destroy buffer!");
    }
}
