use super::buffer::Buffer;
use crate::VkInstance;
use ash::{version::DeviceV1_0, vk};
use std::ptr;

pub struct DescriptorInfo {
    pub layouts: Vec<vk::DescriptorSetLayout>,
    pub buffer_info: Vec<vk::DescriptorBufferInfo>,
    pub buffer: Buffer,
}

impl DescriptorInfo {
    pub fn new(
        bindings: Vec<vk::DescriptorSetLayoutBinding>,
        buffer: Buffer,
        vulkan: &VkInstance,
    ) -> DescriptorInfo {
        let descriptor_layouts = Self::create_descriptor_set_layout(&vulkan, bindings.clone());
        let buffer_info = vec![vk::DescriptorBufferInfo {
            buffer: buffer.buffer,
            offset: 0,
            range: buffer.size as u64,
        }];

        DescriptorInfo {
            layouts: descriptor_layouts,
            buffer_info,
            buffer,
        }
    }

    fn create_descriptor_set_layout(
        vulkan: &VkInstance,
        bindings: Vec<vk::DescriptorSetLayoutBinding>,
    ) -> Vec<vk::DescriptorSetLayout> {
        let ubo_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            flags: vk::DescriptorSetLayoutCreateFlags::empty(),
            binding_count: bindings.len() as u32,
            p_bindings: bindings.as_ptr(),
            ..Default::default()
        };

        unsafe {
            vec![vulkan
                .device
                .create_descriptor_set_layout(&ubo_layout_create_info, None)
                .expect("Failed to create Descriptor Set Layout!")]
        }
    }
    pub fn build(
        &self,
        vulkan: &VkInstance,
        descriptor_pool: &vk::DescriptorPool,
        amount: usize,
    ) -> Vec<vk::DescriptorSet> {
        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            descriptor_pool: *descriptor_pool,
            descriptor_set_count: amount as u32,
            p_set_layouts: self.layouts.as_ptr(),
            ..Default::default()
        };

        let descriptor_sets = unsafe {
            vulkan
                .device
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .expect("Failed to allocate descriptor sets!")
        };

        let descriptor_write_sets = [vk::WriteDescriptorSet {
            s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
            dst_set: descriptor_sets[0],
            dst_binding: 0,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            p_image_info: ptr::null(),
            p_buffer_info: self.buffer_info.as_ptr(),
            ..Default::default()
        }];

        unsafe {
            vulkan
                .device
                .update_descriptor_sets(&descriptor_write_sets, &[]);
        }
        descriptor_sets
    }

    pub fn destroy(&mut self, vulkan: &VkInstance) {
        for layout in &self.layouts {
            unsafe {
                vulkan.device.destroy_descriptor_set_layout(*layout, None);
                self.buffer.destroy(&vulkan);
            }
        }
    }
}
