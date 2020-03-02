use vulkan::{
    modules::swapchain::Swapchain,
    offset_of,
    utilities::{Buffer, Image, Shader, VertexDescriptor},
    VkInstance,
};

use ash::{version::DeviceV1_0, vk};
use cgmath::{Deg, Matrix4, Point3, SquareMatrix, Vector3};
use image::GenericImageView;
use std::default::Default;
use std::ffi::CString;
use std::mem::{self, align_of};
use std::path::Path;
use std::ptr;

#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 3],
    pub tex_coord: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct UniformBufferObject {
    model: Matrix4<f32>,
    view: Matrix4<f32>,
    proj: Matrix4<f32>,
}

//Creates a new pipeline
pub fn create_pipeline(
    swapchain: &Swapchain,
    renderpass: vk::RenderPass,
    vulkan: &VkInstance,
) -> (
    vk::Pipeline,
    vk::PipelineLayout,
    VertexDescriptor,
    Vec<vk::DescriptorSet>,
) {
    let vertex_descriptor = VertexDescriptor {
        binding_descriptor: vec![vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }],
        attribute_descriptor: vec![
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Vertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, color) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Vertex, tex_coord) as u32,
            },
        ],
        size: 3 * std::mem::size_of::<Vertex>() as u64,
        align: align_of::<Vertex>() as u64,
    };

    let descriptor_len = vertex_descriptor.attribute_descriptor.len() as u32;
    let binding_len = vertex_descriptor.binding_descriptor.len() as u32;

    let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo {
        vertex_attribute_description_count: descriptor_len,
        p_vertex_attribute_descriptions: vertex_descriptor.attribute_descriptor.as_ptr(),
        vertex_binding_description_count: binding_len,
        p_vertex_binding_descriptions: vertex_descriptor.binding_descriptor.as_ptr(),
        ..Default::default()
    };
    let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        ..Default::default()
    };
    let viewports = [vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: swapchain.extent.width as f32,
        height: swapchain.extent.height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    }];
    let scissors = [vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: swapchain.extent,
    }];
    let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
        .scissors(&scissors)
        .viewports(&viewports);

    let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        line_width: 1.0,
        polygon_mode: vk::PolygonMode::FILL,
        ..Default::default()
    };
    let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        ..Default::default()
    };
    let noop_stencil_state = vk::StencilOpState {
        fail_op: vk::StencilOp::KEEP,
        pass_op: vk::StencilOp::KEEP,
        depth_fail_op: vk::StencilOp::KEEP,
        compare_op: vk::CompareOp::ALWAYS,
        ..Default::default()
    };
    let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
        depth_test_enable: 1,
        depth_write_enable: 1,
        depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
        front: noop_stencil_state,
        back: noop_stencil_state,
        max_depth_bounds: 1.0,
        ..Default::default()
    };
    let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
        blend_enable: 0,
        src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
        dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::ZERO,
        dst_alpha_blend_factor: vk::BlendFactor::ZERO,
        alpha_blend_op: vk::BlendOp::ADD,
        color_write_mask: vk::ColorComponentFlags::all(),
    }];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op(vk::LogicOp::CLEAR)
        .attachments(&color_blend_attachment_states);

    let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state_info =
        vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);

    let (vertex_shader_module, fragment_shader_module) = Shader {
        vertex_shader: "examples/texture/shaders/spv/shader-textures.vert.spv",
        fragment_shader: "examples/texture/shaders/spv/shader-textures.frag.spv",
    }
    .load(&vulkan);

    let shader_entry_name = CString::new("main").unwrap();
    let shaders = &[
        vk::PipelineShaderStageCreateInfo {
            module: vertex_shader_module,
            p_name: shader_entry_name.as_ptr(),
            stage: vk::ShaderStageFlags::VERTEX,
            ..Default::default()
        },
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            module: fragment_shader_module,
            p_name: shader_entry_name.as_ptr(),
            stage: vk::ShaderStageFlags::FRAGMENT,
            ..Default::default()
        },
    ];

    let uniform_data = create_uniform_data(&swapchain);

    let buffer_create_info = vk::BufferCreateInfo {
        size: std::mem::size_of_val(&uniform_data) as u64,
        usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };

    let mut uniform_buffer = vulkan.create_buffer(buffer_create_info);

    uniform_buffer.copy_to_buffer_dynamic(
        align_of::<UniformBufferObject>() as u64,
        &[uniform_data],
        &vulkan,
    );

    let descriptor_info = vec![
        vk::DescriptorSetLayoutBinding {
            // transform uniform
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers: ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            // sampler uniform
            binding: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: ptr::null(),
        },
    ];

    //Create texture image

    let texture = create_texture(&Path::new("examples/assets/texture.jpg"), &vulkan);

    let imageview_info = vk::ImageViewCreateInfo {
        s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
        view_type: vk::ImageViewType::TYPE_2D,
        format: vk::Format::R8G8B8A8_UNORM,
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
        image: texture.image,
        ..Default::default()
    };

    let sampler_create_info = vk::SamplerCreateInfo {
        s_type: vk::StructureType::SAMPLER_CREATE_INFO,
        mag_filter: vk::Filter::LINEAR,
        min_filter: vk::Filter::LINEAR,
        mipmap_mode: vk::SamplerMipmapMode::LINEAR,
        address_mode_u: vk::SamplerAddressMode::REPEAT,
        address_mode_v: vk::SamplerAddressMode::REPEAT,
        address_mode_w: vk::SamplerAddressMode::REPEAT,
        mip_lod_bias: 0.0,
        anisotropy_enable: vk::TRUE,
        max_anisotropy: 16.0,
        ..Default::default()
    };

    let sampler = vulkan.create_texture_sampler(sampler_create_info);
    let image_view = unsafe {
        vulkan
            .device
            .create_image_view(&imageview_info, None)
            .expect("Failed to create Image View!")
    };
    //Create uniform buffer

    let descriptor_pool = create_descriptor_pool(&vulkan, swapchain.image_views.len() as u32);
    let (descriptor_set, descriptor_layout) = create_descriptors(
        descriptor_info,
        uniform_buffer,
        sampler,
        image_view,
        &descriptor_pool,
        &vulkan,
    );

    let layout_create_info =
        vk::PipelineLayoutCreateInfo::builder().set_layouts(&descriptor_layout);

    //Create pipeline stuff
    let pipeline_layout = unsafe {
        vulkan
            .device
            .create_pipeline_layout(&layout_create_info, None)
            .unwrap()
    };

    let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(shaders)
        .vertex_input_state(&vertex_input_state_info)
        .input_assembly_state(&vertex_input_assembly_state_info)
        .viewport_state(&viewport_state_info)
        .rasterization_state(&rasterization_info)
        .multisample_state(&multisample_state_info)
        .depth_stencil_state(&depth_state_info)
        .color_blend_state(&color_blend_state)
        .dynamic_state(&dynamic_state_info)
        .layout(pipeline_layout)
        .render_pass(renderpass);

    let pipeline = unsafe {
        vulkan
            .device
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[graphic_pipeline_info.build()],
                None,
            )
            .expect("Unable to create graphics pipeline")
    };

    //Destoy shader modules
    unsafe {
        vulkan
            .device
            .destroy_shader_module(vertex_shader_module, None);
        vulkan
            .device
            .destroy_shader_module(fragment_shader_module, None);
    }

    (
        pipeline[0],
        pipeline_layout,
        vertex_descriptor,
        descriptor_set,
    )
}

pub fn create_uniform_data(swapchain: &Swapchain) -> UniformBufferObject {
    UniformBufferObject {
        model: Matrix4::<f32>::identity(),
        view: Matrix4::look_at(
            Point3::new(2.0, 2.0, 2.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        ),
        proj: cgmath::perspective(
            Deg(45.0),
            swapchain.extent.width as f32 / swapchain.extent.height as f32,
            0.1,
            10.0,
        ),
    }
}

pub fn create_texture(image_path: &Path, vulkan: &VkInstance) -> Image {
    let mut image_object = image::open(image_path).unwrap(); // this function is slow in debug mode.
    image_object = image_object.flipv();
    let (image_width, image_height) = (image_object.width(), image_object.height());
    let image_size =
        (std::mem::size_of::<u8>() as u32 * image_width * image_height * 4) as vk::DeviceSize;
    let image_data = match &image_object {
        image::DynamicImage::ImageLuma8(_)
        | image::DynamicImage::ImageBgr8(_)
        | image::DynamicImage::ImageRgb8(_) => image_object.to_rgba().into_raw(),
        image::DynamicImage::ImageLumaA8(_)
        | image::DynamicImage::ImageBgra8(_)
        | image::DynamicImage::ImageRgba8(_) => image_object.raw_pixels(),
    };

    if image_size <= 0 {
        panic!("Failed to load texture image!")
    }

    let image_buffer = vk::BufferCreateInfo {
        size: image_size,
        usage: vk::BufferUsageFlags::TRANSFER_SRC,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };

    let mut buffer = vulkan.create_buffer(image_buffer);
    buffer.copy_buffer::<u8>(image_size, &image_data, &vulkan);

    let image_create_info = vk::ImageCreateInfo {
        s_type: vk::StructureType::IMAGE_CREATE_INFO,
        image_type: vk::ImageType::TYPE_2D,
        format: vk::Format::R8G8B8A8_UNORM,
        extent: vk::Extent3D {
            width: image_width,
            height: image_height,
            depth: 1,
        },
        mip_levels: 1,
        array_layers: 1,
        samples: vk::SampleCountFlags::TYPE_1,
        tiling: vk::ImageTiling::OPTIMAL,
        usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };

    let (image, memory) = Image::create_image(
        image_create_info,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        &vulkan,
    );
    vulkan.transition_image_layout(
        image,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    );
    vulkan.copy_buffer_to_image(buffer.buffer, image, image_width, image_height);

    vulkan.transition_image_layout(
        image,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );

    buffer.destroy(&vulkan);

    Image { image, memory }
}

fn create_descriptors(
    bindings: Vec<vk::DescriptorSetLayoutBinding>,
    buffer: Buffer,
    sampler: vk::Sampler,
    image_view: vk::ImageView,
    descriptor_pool: &vk::DescriptorPool,
    vulkan: &VkInstance,
) -> (Vec<vk::DescriptorSet>, Vec<vk::DescriptorSetLayout>) {
    let ubo_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: bindings.len() as u32,
        p_bindings: bindings.as_ptr(),
        ..Default::default()
    };
    let layouts = unsafe {
        vec![vulkan
            .device
            .create_descriptor_set_layout(&ubo_layout_create_info, None)
            .expect("Failed to create Descriptor Set Layout!")]
    };

    let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
        descriptor_pool: *descriptor_pool,
        descriptor_set_count: 1 as u32,
        p_set_layouts: layouts.as_ptr(),
        ..Default::default()
    };

    let descriptor_sets = unsafe {
        vulkan
            .device
            .allocate_descriptor_sets(&descriptor_set_allocate_info)
            .expect("Failed to allocate descriptor sets!")
    };

    let descriptor_image_infos = [vk::DescriptorImageInfo {
        sampler: sampler,
        image_view: image_view,
        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    }];

    let buffer_info = vec![vk::DescriptorBufferInfo {
        buffer: buffer.buffer,
        offset: 0,
        range: buffer.size as u64,
    }];

    let descriptor_write_sets = [
        vk::WriteDescriptorSet {
            // transform uniform
            s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
            p_next: ptr::null(),
            dst_set: descriptor_sets[0],
            dst_binding: 0,
            dst_array_element: 0,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            p_image_info: ptr::null(),
            p_buffer_info: buffer_info.as_ptr(),
            p_texel_buffer_view: ptr::null(),
        },
        vk::WriteDescriptorSet {
            // sampler uniform
            s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
            p_next: ptr::null(),
            dst_set: descriptor_sets[0],
            dst_binding: 1,
            dst_array_element: 0,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            p_image_info: descriptor_image_infos.as_ptr(),
            p_buffer_info: ptr::null(),
            p_texel_buffer_view: ptr::null(),
        },
    ];

    unsafe {
        vulkan
            .device
            .update_descriptor_sets(&descriptor_write_sets, &[]);
    }

    (descriptor_sets, layouts)
}

pub fn create_descriptor_pool(vulkan: &VkInstance, size: u32) -> vk::DescriptorPool {
    let pool_sizes = [
        vk::DescriptorPoolSize {
            // transform descriptor pool
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: size,
        },
        vk::DescriptorPoolSize {
            // sampler descriptor pool
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: size,
        },
    ];

    let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::DescriptorPoolCreateFlags::empty(),
        max_sets: size,
        pool_size_count: pool_sizes.len() as u32,
        p_pool_sizes: pool_sizes.as_ptr(),
    };

    unsafe {
        vulkan
            .device
            .create_descriptor_pool(&descriptor_pool_create_info, None)
            .expect("Failed to create Descriptor Pool!")
    }
}
