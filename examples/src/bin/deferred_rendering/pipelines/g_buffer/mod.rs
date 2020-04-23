mod framebuffers;
mod pipeline;

use examples::utils::{
    gltf_importer::{MaterialRaw, Scene, Vertex},
    Camera, CameraRaw,
};
use std::sync::Arc;
use vulkan::{
    modules::swapchain::Swapchain,
    offset_of,
    prelude::*,
    utilities::{as_byte_slice, Shader},
    Buffer, Context, Descriptor, DescriptorSet, Image, VkThread,
};

pub struct GBuffer {
    pub color: Image,
    pub depth: Image,
    pub position: Image,
    pub normal: Image,
}

impl GBuffer {
    pub fn new(scene: &Scene, swapchain: &Swapchain, vulkan: &VkThread) {
        let context = vulkan.context();

        let (width, height) = (swapchain.extent.width, swapchain.extent.height);

        let color = create_image(
            vk::Format::R8G8B8A8_UNORM,
            width,
            height,
            context.clone()
        );

        let normal = create_image(
            vk::Format::R16G16B16A16_SFLOAT,
            width,
            height,
            context.clone()
        );

        let position = create_image(
            vk::Format::R16G16B16A16_SFLOAT,
            width,
            height,
            context.clone()
        );

        let depth = examples::create_depth_resources(&swapchain, context.clone());
    }
}

fn create_image(
    format: vk::Format,
    width: u32,
    height: u32,
    context: Arc<Context>,
) -> Image {
    let mut image = Image::create_image(
        vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            image_type: vk::ImageType::TYPE_2D,
            format,
            extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SampleCountFlags::TYPE_1,
            tiling: vk::ImageTiling::OPTIMAL,
            usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        },
        vk_mem::MemoryUsage::GpuOnly,
        context.clone(),
    );

    image.attach_view(vk::ImageViewCreateInfo {
        view_type: vk::ImageViewType::TYPE_2D,
        format,
        components: vk::ComponentMapping {
            r: vk::ComponentSwizzle::IDENTITY,
            g: vk::ComponentSwizzle::IDENTITY,
            b: vk::ComponentSwizzle::IDENTITY,
            a: vk::ComponentSwizzle::IDENTITY,
        },
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        },
        image: image.image(),
        ..Default::default()
    });
    image
}
