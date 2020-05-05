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
    pub max_points_per_light: u32,
    pub tile_size: u32,
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
                offset: offset_of!(Self, max_points_per_light) as _,
                size: mem::size_of::<u32>(),
            },
            vk::SpecializationMapEntry {
                constant_id: 3,
                offset: offset_of!(Self, tile_size) as _,
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

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct ForwardPushConstant {
    pub transform: cgmath::Matrix4<f32>,
    pub screen_width: u32,
    pub screen_height: u32,
    pub row_count: u32,
    pub column_count: u32,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct ComputePushConstant {
    pub screen_width: u32,
    pub screen_height: u32,
    pub row_count: u32,
    pub column_count: u32,
}

#[allow(dead_code)]
pub const MAX_POINT_LIGHT_PER_TILE: u32 = 1023;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LightVisibility {
    pub count: u32,
    pub indicies: [u32; MAX_POINT_LIGHT_PER_TILE as usize],
}
