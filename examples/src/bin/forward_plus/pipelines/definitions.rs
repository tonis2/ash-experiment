use std::mem;
use vulkan::{offset_of, prelude::*, utilities::as_byte_slice};

#[allow(dead_code)]
pub struct ForwardConstants {
    pub materials_amount: u32,
    pub textures_amount: u32,
    pub lights_amount: u32,
}
#[allow(dead_code)]
impl ForwardConstants {
    fn specialization_map_entries(&self) -> [vk::SpecializationMapEntry; 3] {
        // Each shader constant of a shader stage corresponds to one map entry
        [
            vk::SpecializationMapEntry {
                constant_id: 0,
                offset: offset_of!(Self, materials_amount) as _,
                size: mem::size_of::<u32>(),
            },
            vk::SpecializationMapEntry {
                constant_id: 1,
                offset: offset_of!(Self, textures_amount) as _,
                size: mem::size_of::<u32>(),
            },
            vk::SpecializationMapEntry {
                constant_id: 2,
                offset: offset_of!(Self, lights_amount) as _,
                size: mem::size_of::<u32>(),
            },
        ]
    }

    pub fn get_info(&self) -> vk::SpecializationInfo {
        unsafe {
            vk::SpecializationInfo::builder()
                .map_entries(&self.specialization_map_entries())
                .data(as_byte_slice(&self))
                .build()
        }
    }
}

#[allow(dead_code)]
pub struct ComputeConstants {
    pub lights_amount: u32,
    pub tile_numns: cgmath::Vector2<i32>,
    pub viewport_size: cgmath::Vector2<i32>,
}

#[allow(dead_code)]
impl ComputeConstants {
    fn specialization_map_entries(&self) -> [vk::SpecializationMapEntry; 3] {
        // Each shader constant of a shader stage corresponds to one map entry
        [
            vk::SpecializationMapEntry {
                constant_id: 0,
                offset: offset_of!(Self, lights_amount) as _,
                size: mem::size_of::<u32>(),
            },
            vk::SpecializationMapEntry {
                constant_id: 1,
                offset: offset_of!(Self, tile_numns) as _,
                size: mem::size_of::<cgmath::Vector2<i32>>(),
            },
            vk::SpecializationMapEntry {
                constant_id: 2,
                offset: offset_of!(Self, viewport_size) as _,
                size: mem::size_of::<cgmath::Vector2<i32>>(),
            },
        ]
    }

    pub fn get_info(&self) -> vk::SpecializationInfo {
        unsafe {
            vk::SpecializationInfo::builder()
                .map_entries(&self.specialization_map_entries())
                .data(as_byte_slice(&self))
                .build()
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct PushTransform {
    pub transform: cgmath::Matrix4<f32>,
}
