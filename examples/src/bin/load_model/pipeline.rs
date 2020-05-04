use vulkan::{
    Swapchain, offset_of, prelude::*, Buffer, Context, Descriptor,
    DescriptorSet, Image, Shader, VkThread,
};

use examples::utils::Camera;
use image::GenericImageView;
use std::{default::Default, ffi::CString, mem, path::Path, ptr, sync::Arc};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct PushTransform {
    pub model: cgmath::Matrix4<f32>,
}

impl PushTransform {
    pub fn new(transform: cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Basis3<f32>>) -> Self {
        Self {
            model: transform.into(),
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub color: [f32; 4],
    pub tex_coord: [f32; 2],
}

pub struct Pipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub texture: (Image, u32),
    pub depth_image: Image,
    pub camera_buffer: Buffer,
    pub camera: Camera,

    pub pipeline_descriptor: Descriptor,
    pub renderpass: vk::RenderPass,

    context: Arc<Context>,
}

impl Pipeline {
    //Creates a new pipeline
    pub fn new(swapchain: &Swapchain, vulkan: &VkThread) -> Pipeline {
        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&[vk::VertexInputBindingDescription {
                binding: 0,
                stride: mem::size_of::<Vertex>() as u32,
                input_rate: vk::VertexInputRate::VERTEX,
            }])
            .vertex_attribute_descriptions(&[
                vk::VertexInputAttributeDescription {
                    binding: 0,
                    location: 0,
                    format: vk::Format::R32G32B32_SFLOAT,
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
                    format: vk::Format::R32G32B32_SFLOAT,
                    offset: offset_of!(Vertex, tex_coord) as u32,
                },
            ])
            .build();
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

        //Create texture image

        let (texture, mip_levels) = create_texture(&Path::new("assets/chalet.jpg"), &vulkan);

        //Create uniform buffer

        let camera = Camera::new(cgmath::Point3::new(0.0, 0.0, 0.0), 5.0, 1.3);

        let camera_buffer = Buffer::new_mapped_basic(
            std::mem::size_of_val(&camera.raw()) as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk_mem::MemoryUsage::CpuOnly,
            vulkan.context(),
        );

        camera_buffer.upload_to_buffer(&[camera.raw()], 0);

        let pipeline_descriptor = Descriptor::new(
            vec![
                DescriptorSet {
                    bind_index: 0,
                    flag: vk::ShaderStageFlags::VERTEX,
                    bind_type: vk::DescriptorType::UNIFORM_BUFFER,
                    buffer_info: Some(vec![vk::DescriptorBufferInfo {
                        buffer: camera_buffer.buffer,
                        offset: 0,
                        range: camera_buffer.size,
                    }]),
                    ..Default::default()
                },
                DescriptorSet {
                    bind_index: 1,
                    flag: vk::ShaderStageFlags::FRAGMENT,
                    bind_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    image_info: Some(vec![vk::DescriptorImageInfo {
                        sampler: texture.sampler(),
                        image_view: texture.view(),
                        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    }]),
                    ..Default::default()
                },
            ],
            vulkan.context(),
        );

        let layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&[pipeline_descriptor.layout])
            .push_constant_ranges(&[vk::PushConstantRange {
                stage_flags: vk::ShaderStageFlags::VERTEX,
                size: mem::size_of::<PushTransform>() as u32,
                offset: 0,
            }])
            .build();

        //Create pipeline stuff
        let pipeline_layout = unsafe {
            vulkan
                .device()
                .create_pipeline_layout(&layout_create_info, None)
                .unwrap()
        };
        let renderpass = create_render_pass(&swapchain, vulkan);

        let shader_name = CString::new("main").unwrap();
        let pipeline = unsafe {
            vulkan
                .device()
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[vk::GraphicsPipelineCreateInfo::builder()
                        .stages(&[
                            Shader::new(
                                &Path::new("src/bin/load_model/shaders/model.vert.spv"),
                                vk::ShaderStageFlags::VERTEX,
                                &shader_name,
                                vulkan.context(),
                            )
                            .info(),
                            Shader::new(
                                &Path::new("src/bin/load_model/shaders/model.frag.spv"),
                                vk::ShaderStageFlags::FRAGMENT,
                                &shader_name,
                                vulkan.context(),
                            )
                            .info(),
                        ])
                        .vertex_input_state(&vertex_input_state_info)
                        .input_assembly_state(&vertex_input_assembly_state_info)
                        .viewport_state(&viewport_state_info)
                        .rasterization_state(&rasterization_info)
                        .multisample_state(&multisample_state_info)
                        .depth_stencil_state(&depth_state_info)
                        .color_blend_state(&color_blend_state)
                        .dynamic_state(&dynamic_state_info)
                        .layout(pipeline_layout)
                        .render_pass(renderpass)
                        .build()],
                    None,
                )
                .expect("Unable to create graphics pipeline")
        };

        let depth_image = examples::create_depth_resources(&swapchain, vulkan.context());

        Pipeline {
            pipeline: pipeline[0],
            layout: pipeline_layout,
            texture: (texture, mip_levels),
            depth_image,
            context: vulkan.context(),
            camera,
            camera_buffer,
            renderpass,
            pipeline_descriptor,
        }
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.context.wait_idle();
            self.context.device.destroy_pipeline(self.pipeline, None);
            self.context
                .device
                .destroy_pipeline_layout(self.layout, None);

            self.context
                .device
                .destroy_render_pass(self.renderpass, None);
        }
    }
}

pub fn create_texture(image_path: &Path, vulkan: &VkThread) -> (Image, u32) {
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
    let mip_levels = ((::std::cmp::max(image_width, image_height) as f32)
        .log2()
        .floor() as u32)
        + 1;

    if image_size <= 0 {
        panic!("Failed to load texture image!")
    }

    let buffer = Buffer::new_mapped_basic(
        image_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk_mem::MemoryUsage::CpuOnly,
        vulkan.context(),
    );

    buffer.upload_to_buffer::<u8>(&image_data, 0);

    let mut image = Image::create_image(
        vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            image_type: vk::ImageType::TYPE_2D,
            format: vk::Format::R8G8B8A8_UNORM,
            extent: vk::Extent3D {
                width: image_width,
                height: image_height,
                depth: 1,
            },
            mip_levels,
            array_layers: 1,
            samples: vk::SampleCountFlags::TYPE_1,
            tiling: vk::ImageTiling::OPTIMAL,
            usage: vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::SAMPLED,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        },
        vk_mem::MemoryUsage::GpuOnly,
        vulkan.context(),
    );

    vulkan.apply_pipeline_barrier(
        vk::PipelineStageFlags::TOP_OF_PIPE,
        vk::PipelineStageFlags::TRANSFER,
        vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image: image.image(),
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        },
    );

    let buffer_image_regions = vec![vk::BufferImageCopy {
        image_subresource: vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        },
        image_extent: vk::Extent3D {
            width: image_width,
            height: image_height,
            depth: 1,
        },
        buffer_offset: 0,
        buffer_image_height: 0,
        buffer_row_length: 0,
        image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
    }];

    vulkan.copy_buffer_to_image(buffer.buffer, image.image(), buffer_image_regions);

    vulkan.generate_mipmaps(image.image(), image_width, image_height, mip_levels);

    image.attach_view(vk::ImageViewCreateInfo {
        s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
        view_type: vk::ImageViewType::TYPE_2D,
        format: vk::Format::R8G8B8A8_UNORM,
        image: image.image(),
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
        ..Default::default()
    });

    image.attach_sampler(vk::SamplerCreateInfo {
        s_type: vk::StructureType::SAMPLER_CREATE_INFO,
        mag_filter: vk::Filter::LINEAR,
        min_filter: vk::Filter::LINEAR,
        mipmap_mode: vk::SamplerMipmapMode::LINEAR,
        address_mode_u: vk::SamplerAddressMode::REPEAT,
        address_mode_v: vk::SamplerAddressMode::REPEAT,
        address_mode_w: vk::SamplerAddressMode::REPEAT,
        max_lod: mip_levels as f32,
        mip_lod_bias: 0.0,
        anisotropy_enable: vk::TRUE,
        max_anisotropy: 16.0,
        ..Default::default()
    });

    (image, mip_levels)
}

pub fn create_render_pass(swapchain: &Swapchain, vulkan: &VkThread) -> vk::RenderPass {
    let color_attachment = vk::AttachmentDescription {
        flags: vk::AttachmentDescriptionFlags::empty(),
        format: swapchain.format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
    };

    let depth_format = vulkan.context.find_depth_format(
        &[
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ],
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    );

    let depth_attachment = vk::AttachmentDescription {
        flags: vk::AttachmentDescriptionFlags::empty(),
        format: depth_format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::DONT_CARE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    };

    let color_attachment_ref = vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    };

    let depth_attachment_ref = vk::AttachmentReference {
        attachment: 1,
        layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    };

    let subpasses = [vk::SubpassDescription {
        color_attachment_count: 1,
        p_color_attachments: &color_attachment_ref,
        p_depth_stencil_attachment: &depth_attachment_ref,
        flags: vk::SubpassDescriptionFlags::empty(),
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
        input_attachment_count: 0,
        p_input_attachments: ptr::null(),
        p_resolve_attachments: ptr::null(),
        preserve_attachment_count: 0,
        p_preserve_attachments: ptr::null(),
    }];

    let render_pass_attachments = [color_attachment, depth_attachment];

    let subpass_dependencies = [vk::SubpassDependency {
        src_subpass: vk::SUBPASS_EXTERNAL,
        dst_subpass: 0,
        src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        src_access_mask: vk::AccessFlags::empty(),
        dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
            | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        dependency_flags: vk::DependencyFlags::empty(),
    }];

    let renderpass_create_info = vk::RenderPassCreateInfo {
        s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
        flags: vk::RenderPassCreateFlags::empty(),
        p_next: ptr::null(),
        attachment_count: render_pass_attachments.len() as u32,
        p_attachments: render_pass_attachments.as_ptr(),
        subpass_count: subpasses.len() as u32,
        p_subpasses: subpasses.as_ptr(),
        dependency_count: subpass_dependencies.len() as u32,
        p_dependencies: subpass_dependencies.as_ptr(),
    };

    unsafe {
        vulkan
            .device()
            .create_render_pass(&renderpass_create_info, None)
            .expect("Failed to create render pass!")
    }
}
