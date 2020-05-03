use vulkan::{
    prelude::*, utilities::as_byte_slice, Buffer, Descriptor, DescriptorSet, Framebuffer, Image,
    Pipeline, Renderpass, Shader, Swapchain, VkThread,
};

use std::{default::Default, ffi::CString, path::Path};

use super::definitions::{PushTransform, SpecializationData};
use examples::utils::{
    gltf_importer::{Light, MaterialRaw, Scene, Vertex},
    Camera, CameraRaw,
};
use std::mem;

pub struct Pipes {
    pub camera: Camera,
    pub camera_buffer: Buffer,
    pub materials_buffer: Buffer,

    //Depth texture
    pub depth_image: Image,
    pub depth_descriptor: Descriptor,

    pub pipelines: Pipeline,

    _empty_image: Image,
}

pub fn new(scene: &Scene, swapchain: &Swapchain, vulkan: &VkThread) {
    let light_buffer = vulkan.create_gpu_buffer(
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        &scene
            .get_lights()
            .map_or(vec![Light::default()], |lights| lights.clone()),
    );

    let light_bindings: Vec<vk::DescriptorBufferInfo> = scene.get_lights().map_or(
        vec![vk::DescriptorBufferInfo {
            buffer: light_buffer.buffer,
            offset: 0,
            range: light_buffer.size,
        }],
        |lights| {
            lights
                .iter()
                .enumerate()
                .map(|(index, _material)| vk::DescriptorBufferInfo {
                    buffer: light_buffer.buffer,
                    offset: (index * mem::size_of::<Light>() as usize) as u64,
                    range: light_buffer.size,
                })
                .collect()
        },
    );

    //Create material buffers
    let material_buffer = vulkan.create_gpu_buffer(
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        &scene
            .get_materials()
            .map_or(vec![MaterialRaw::default()], |material| material),
    );

    let material_bindings: Vec<vk::DescriptorBufferInfo> = scene.get_materials().map_or(
        vec![vk::DescriptorBufferInfo {
            buffer: material_buffer.buffer,
            offset: 0,
            range: material_buffer.size,
        }],
        |materials| {
            materials
                .iter()
                .enumerate()
                .map(|(index, _material)| vk::DescriptorBufferInfo {
                    buffer: material_buffer.buffer,
                    offset: (index * mem::size_of::<MaterialRaw>() as usize) as u64,
                    range: material_buffer.size,
                })
                .collect()
        },
    );
}
