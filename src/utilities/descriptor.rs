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

impl Descriptor {
    pub fn new(
        bindings: Vec<vk::DescriptorSetLayoutBinding>,
        write_set: Vec<vk::WriteDescriptorSet>,
        context: Arc<Context>,
    ) -> Self {
        let pool_sizes: Vec<vk::DescriptorPoolSize> = bindings
            .iter()
            .map(|binding| vk::DescriptorPoolSize {
                ty: binding.descriptor_type,
                descriptor_count: context.image_count,
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

        let layouts = unsafe {
            vec![context
                .device
                .create_descriptor_set_layout(
                    &vk::DescriptorSetLayoutCreateInfo {
                        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
                        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
                        binding_count: bindings.len() as u32,
                        p_bindings: bindings.as_ptr(),
                        ..Default::default()
                    },
                    None,
                )
                .expect("Failed to create Descriptor Set Layout!")]
        };
        let mut write_sets = write_set.clone();
        let descriptor_sets = unsafe {
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

        for descriptor in write_sets.iter_mut() {
            descriptor.dst_set = descriptor_sets[0]
        }

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
