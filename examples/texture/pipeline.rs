use vulkan::{
    modules::swapchain::Swapchain,
    offset_of,
    utilities::{DescriptorInfo, Image, Shader, VertexDescriptor},
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
    pub color: [f32; 4],
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
    DescriptorInfo,
    Image,
) {
    let vertex_descriptor = VertexDescriptor {
        binding_descriptor: vec![vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }],
        attribute_descriptor: vec![
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, color) as u32,
            },
        ],
        size: 3 * std::mem::size_of::<Vertex>() as u64,
        align: align_of::<Vertex>() as u64,
    };

    let descriptor_len = vertex_descriptor.attribute_descriptor.len() as u32;
    let binding_len = vertex_descriptor.binding_descriptor.len() as u32;

    unsafe {
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
            vertex_shader: "examples/load_model/shaders/spv/shader-ubo.vert.spv",
            fragment_shader: "examples/load_model/shaders/spv/shader-ubo.frag.spv",
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

        uniform_buffer.copy_to_buffer(
            align_of::<UniformBufferObject>() as u64,
            &[uniform_data],
            &vulkan,
        );

        let uniform_descriptor = DescriptorInfo::new(
            vec![vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                p_immutable_samplers: ptr::null(),
            }],
            uniform_buffer,
            &vulkan,
        );

        let layout_create_info =
            vk::PipelineLayoutCreateInfo::builder().set_layouts(&uniform_descriptor.layouts);

        let pipeline_layout = vulkan
            .device
            .create_pipeline_layout(&layout_create_info, None)
            .unwrap();

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

        let pipeline = vulkan
            .device
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[graphic_pipeline_info.build()],
                None,
            )
            .expect("Unable to create graphics pipeline");

        //Destoy shader modules
        vulkan
            .device
            .destroy_shader_module(vertex_shader_module, None);
        vulkan
            .device
            .destroy_shader_module(fragment_shader_module, None);

        let texture = create_texture(
            &Path::new("examples/assets/texture.jpg"),
            &vulkan,
            &swapchain,
        );

        let imageview_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            view_type: vk::ImageViewType::TYPE_2D,
            format: swapchain.format,
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
        let texture_image_view = vulkan.create_image_view(imageview_info);

        (
            pipeline[0],
            pipeline_layout,
            vertex_descriptor,
            uniform_descriptor,
            texture,
        )
    }
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

pub fn create_texture(image_path: &Path, vulkan: &VkInstance, swapchain: &Swapchain) -> Image {
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
    buffer.copy_to_buffer(image_size, &image_data[..], &vulkan);

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

    vulkan.copy_buffer_to_image(buffer.buffer, image, image_width, image_height);

    Image { image, memory }
}
