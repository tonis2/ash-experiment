use crate::Context;
use ash::version::DeviceV1_0;
use ash::vk;
use std::ptr;
use std::sync::Arc;

pub struct Descriptor {
    pub set: vk::DescriptorSet,
    pub layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    context: Arc<Context>,
}
#[derive(Debug, Clone)]
pub struct DescriptorSet {
    pub flag: vk::ShaderStageFlags,
    pub bind_type: vk::DescriptorType,
    pub bind_index: u32,
    pub count: u32,
    pub buffer_info: Option<Vec<vk::DescriptorBufferInfo>>,
    pub image_info: Option<Vec<vk::DescriptorImageInfo>>,
    pub array_element: u32,
}

impl Default for DescriptorSet {
    fn default() -> Self {
        Self {
            flag: vk::ShaderStageFlags::default(),
            bind_type: vk::DescriptorType::default(),
            bind_index: 0,
            count: 1,
            buffer_info: None,
            image_info: None,
            array_element: 0,
        }
    }
}

impl Descriptor {
    //Creates new pipeline descriptor
    pub fn new(sets: Vec<DescriptorSet>, context: Arc<Context>) -> Self {
        let pool_sizes: &Vec<vk::DescriptorPoolSize> = &sets
            .iter()
            .map(|set| vk::DescriptorPoolSize {
                ty: set.bind_type,
                descriptor_count: set.count,
            })
            .collect();

        let pool = unsafe {
            context
                .device
                .create_descriptor_pool(
                    &vk::DescriptorPoolCreateInfo {
                        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
                        p_next: ptr::null(),
                        flags: vk::DescriptorPoolCreateFlags::empty(),
                        max_sets: context.image_count,
                        pool_size_count: pool_sizes.len() as u32,
                        p_pool_sizes: pool_sizes.as_ptr(),
                    },
                    None,
                )
                .expect("Failed to create Descriptor Pool!")
        };

        let bindings: Vec<vk::DescriptorSetLayoutBinding> = sets
            .iter()
            .map(|set| vk::DescriptorSetLayoutBinding {
                binding: set.bind_index,
                descriptor_type: set.bind_type,
                descriptor_count: set.count,
                stage_flags: set.flag,
                ..Default::default()
            })
            .collect();

        let layouts = unsafe {
            vec![context
                .device
                .create_descriptor_set_layout(
                    &vk::DescriptorSetLayoutCreateInfo::builder()
                        .bindings(&bindings)
                        .build(),
                    None,
                )
                .expect("Failed to create Descriptor Set Layout!")]
        };
        let descriptor_sets: Vec<vk::DescriptorSet> = unsafe {
            context
                .device
                .allocate_descriptor_sets(&vk::DescriptorSetAllocateInfo {
                    s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
                    descriptor_pool: pool,
                    descriptor_set_count: 1 as u32,
                    p_set_layouts: layouts.as_ptr(),
                    ..Default::default()
                })
                .expect("Failed to allocate descriptor sets!")
        };
        let write_sets: Vec<vk::WriteDescriptorSet> = sets
            .iter()
            .map(|set| {
                let mut descriptor = vk::WriteDescriptorSet {
                    dst_binding: set.bind_index,
                    dst_array_element: set.array_element,
                    descriptor_count: set.count,
                    descriptor_type: set.bind_type,
                    dst_set: descriptor_sets[0],
                    ..Default::default()
                };
                if set.buffer_info.is_some() {
                    descriptor.p_buffer_info = set.buffer_info.as_ref().unwrap().as_ptr();
                }

                if set.image_info.is_some() {
                    descriptor.p_image_info = set.image_info.as_ref().unwrap().as_ptr();
                }
                descriptor
            })
            .collect();

        unsafe {
            context.device.update_descriptor_sets(&write_sets, &[]);
        }

        Self {
            set: descriptor_sets[0],
            layout: layouts[0],
            pool,
            context,
        }
    }
}

impl Drop for Descriptor {
    fn drop(&mut self) {
        unsafe {
            self.context.wait_idle();
            self.context
                .device
                .destroy_descriptor_set_layout(self.layout, None);
            self.context.device.destroy_descriptor_pool(self.pool, None);
        }
    }
}
