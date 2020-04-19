use std::mem;
use vulkan::{offset_of, prelude::*};

pub struct SpecializationData {
    pub materials_amount: u32,
    pub textures_amount: u32,
}

impl SpecializationData {
    pub fn specialization_map_entries(&self) -> [vk::SpecializationMapEntry; 2] {
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
        ]
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct PushTransform {
    pub transform: cgmath::Matrix4<f32>,
}
