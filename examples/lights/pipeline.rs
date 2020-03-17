use vulkan::{
    modules::swapchain::Swapchain,
    offset_of,
    utilities::{tools::load_shader, Buffer, Image},
    Context, VkInstance,
};

use ash::{version::DeviceV1_0, vk};
use cgmath::{Deg, Matrix4, Point3, Vector3};
use image::GenericImageView;

use std::{default::Default, ffi::CString, mem, path::Path, ptr, sync::Arc};

#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub tex_cord: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct UniformBufferObject {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

pub struct Pipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub descriptors: Vec<vk::DescriptorSet>,
    pub texture: (Image, vk::ImageView, u32),
    pub depth_image: (Image, vk::ImageView),
    pub sampler: vk::Sampler,
    pub uniform_buffer: Buffer,
    pub uniform_transform: UniformBufferObject,

    context: Arc<Context>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_layout: Vec<vk::DescriptorSetLayout>,
}

impl Pipeline {
    //Creates a new pipeline
    pub fn create_pipeline(
        swapchain: &Swapchain,
        renderpass: vk::RenderPass,
        vulkan: &VkInstance,
    ) -> Pipeline {
        let vertex_binding = vec![vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }];
        let vertex_attributes = vec![
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Vertex, tex_cord) as u32,
            },
        ];

        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo {
            vertex_attribute_description_count: vertex_attributes.len() as u32,
            p_vertex_attribute_descriptions: vertex_attributes.as_ptr(),
            vertex_binding_description_count: vertex_binding.len() as u32,
            p_vertex_binding_descriptions: vertex_binding.as_ptr(),
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

        let vertex_shader = load_shader(&Path::new("examples/lights/shaders/lights.vert.spv"));
        let frag_shader = load_shader(&Path::new("examples/lights/shaders/lights.frag.spv"));

        let vertex_shader_module = unsafe {
            vulkan
                .device()
                .create_shader_module(
                    &vk::ShaderModuleCreateInfo::builder().code(&vertex_shader),
                    None,
                )
                .expect("Vertex shader module error")
        };

        let fragment_shader_module = unsafe {
            vulkan
                .device()
                .create_shader_module(
                    &vk::ShaderModuleCreateInfo::builder().code(&frag_shader),
                    None,
                )
                .expect("Fragment shader module error")
        };

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

        let mut imageview_info = vk::ImageViewCreateInfo {
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
            ..Default::default()
        };

        let texture = create_texture(
            &Path::new("examples/assets/texture.jpg"),
            &mut imageview_info,
            &vulkan,
        );

        let sampler_create_info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            max_lod: texture.2 as f32,
            mip_lod_bias: 0.0,
            anisotropy_enable: vk::TRUE,
            max_anisotropy: 16.0,
            ..Default::default()
        };

        let sampler = unsafe {
            vulkan
                .context
                .device
                .create_sampler(&sampler_create_info, None)
                .expect("Failed to create Sampler!")
        };

        //Create uniform buffer

        let uniform_data = create_uniform_data(&swapchain);

        let uniform_buffer = Buffer::new_mapped_basic(
            std::mem::size_of_val(&uniform_data) as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk_mem::MemoryUsage::CpuOnly,
            vulkan.context(),
        );

        uniform_buffer.upload_to_buffer(&[uniform_data], 0);

        let descriptor_pool = create_descriptor_pool(&vulkan, swapchain.image_views.len() as u32);

        let (descriptor_set, descriptor_layout) = create_descriptors(
            descriptor_info,
            &uniform_buffer,
            sampler,
            texture.1,
            &descriptor_pool,
            &vulkan,
        );

        let layout_create_info =
            vk::PipelineLayoutCreateInfo::builder().set_layouts(&descriptor_layout);

        //Create pipeline stuff
        let pipeline_layout = unsafe {
            vulkan
                .device()
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
                .device()
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[graphic_pipeline_info.build()],
                    None,
                )
                .expect("Unable to create graphics pipeline")
        };

        let depth_image = create_depth_resources(&swapchain, &vulkan);

        //Destoy shader modules
        unsafe {
            vulkan
                .device()
                .destroy_shader_module(vertex_shader_module, None);
            vulkan
                .device()
                .destroy_shader_module(fragment_shader_module, None);
        }

        Pipeline {
            pipeline: pipeline[0],
            layout: pipeline_layout,
            texture,
            depth_image,
            sampler,
            descriptors: descriptor_set,
            descriptor_pool,
            descriptor_layout,

            context: vulkan.context(),
            uniform_buffer,
            uniform_transform: uniform_data,
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
            self.context.device.destroy_sampler(self.sampler, None);
            self.context
                .device
                .destroy_descriptor_pool(self.descriptor_pool, None);

            self.context.device.destroy_image_view(self.texture.1, None);
            self.context
                .device
                .destroy_image_view(self.depth_image.1, None);

            self.depth_image.0.destroy(self.context.clone());
            self.texture.0.destroy(self.context.clone());

            self.context
                .device
                .destroy_descriptor_set_layout(self.descriptor_layout[0], None);
        }
    }
}

pub fn create_uniform_data(swapchain: &Swapchain) -> UniformBufferObject {
    UniformBufferObject {
        model: Matrix4::from_angle_z(Deg(90.0)),
        view: Matrix4::look_at(
            Point3::new(2.0, 2.0, 6.5),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.5, 1.0),
        ),
        proj: {
            let mut proj = cgmath::perspective(
                Deg(45.0),
                swapchain.extent.width as f32 / swapchain.extent.height as f32,
                0.1,
                10.0,
            );
            proj[1][1] = proj[1][1] * -1.0;
            proj
        },
    }
}

pub fn create_texture(
    image_path: &Path,
    image_info: &mut vk::ImageViewCreateInfo,
    vulkan: &VkInstance,
) -> (Image, vk::ImageView, u32) {
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

    let image_create_info = vk::ImageCreateInfo {
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
    };

    let image = Image::create_image(
        image_create_info,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vulkan.context(),
    );
    vulkan.transition_image_layout(
        image.image,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        mip_levels,
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

    vulkan.copy_buffer_to_image(buffer.buffer, image.image, buffer_image_regions);

    vulkan.generate_mipmaps(image.image, image_width, image_height, mip_levels);

    image_info.image = image.image;
    let image_view = unsafe {
        vulkan
            .device()
            .create_image_view(&image_info, None)
            .expect("Failed to create Image View!")
    };

    (image, image_view, mip_levels)
}

fn create_descriptors(
    bindings: Vec<vk::DescriptorSetLayoutBinding>,
    buffer: &Buffer,
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
            .device()
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
            .device()
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
        range: buffer.size() as u64 - std::mem::size_of::<UniformBufferObject>() as u64,
    }];

    let descriptor_write_sets = [
        vk::WriteDescriptorSet {
            // transform uniform
            s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
            dst_set: descriptor_sets[0],
            dst_binding: 0,
            dst_array_element: 0,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            p_buffer_info: buffer_info.as_ptr(),
            ..Default::default()
        },
        vk::WriteDescriptorSet {
            // sampler uniform
            s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
            dst_set: descriptor_sets[0],
            dst_binding: 1,
            dst_array_element: 0,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            p_image_info: descriptor_image_infos.as_ptr(),
            ..Default::default()
        },
    ];

    unsafe {
        vulkan
            .device()
            .update_descriptor_sets(&descriptor_write_sets, &[]);
    }

    (descriptor_sets, layouts)
}

//Creates depth image
pub fn create_depth_resources(
    swapchain: &Swapchain,
    vulkan: &VkInstance,
) -> (Image, vk::ImageView) {
    let depth_format = vulkan.find_depth_format(
        &[
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ],
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    );
    let depth_image_info = vk::ImageCreateInfo {
        s_type: vk::StructureType::IMAGE_CREATE_INFO,
        image_type: vk::ImageType::TYPE_2D,
        format: depth_format,
        mip_levels: 1,
        array_layers: 1,
        samples: vk::SampleCountFlags::TYPE_1,
        tiling: vk::ImageTiling::OPTIMAL,
        usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        extent: vk::Extent3D {
            width: swapchain.extent.width,
            height: swapchain.extent.height,
            depth: 1,
        },
        ..Default::default()
    };

    let image = Image::create_image(
        depth_image_info,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vulkan.context(),
    );

    let imageview_info = vk::ImageViewCreateInfo {
        s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
        view_type: vk::ImageViewType::TYPE_2D,
        format: depth_format,
        image: image.image,
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::DEPTH,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        },
        ..Default::default()
    };

    let image_view = unsafe {
        vulkan
            .device()
            .create_image_view(&imageview_info, None)
            .expect("Failed to create Image View!")
    };

    vulkan.transition_image_layout(
        image.image,
        depth_format,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        1,
    );

    (image, image_view)
}

//Creates descriptor pool for uniform buffers and stuff
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
            .device()
            .create_descriptor_pool(&descriptor_pool_create_info, None)
            .expect("Failed to create Descriptor Pool!")
    }
}
