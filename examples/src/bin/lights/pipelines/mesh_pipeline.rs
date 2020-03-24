use vulkan::{
    modules::swapchain::Swapchain,
    offset_of,
    prelude::*,
    utilities::{tools::load_shader, Buffer, Image},
    Context, VkInstance,
};

use cgmath::{Deg, Matrix4, Point3, Vector3};
use image::GenericImageView;
use std::{default::Default, ffi::CString, mem, path::Path, ptr, sync::Arc};

#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub tex_cord: [f32; 2],
    pub normal: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Light {
    pub projection: Matrix4<f32>,
    pub pos: cgmath::Point3<f32>,
    pub color: [f32; 3],
    pub ambient: f32,
    pub specular: f32,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct PushConstantModel {
    pub model: Matrix4<f32>,
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct UniformBufferObject {
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

impl PushConstantModel {
    pub fn new(
        rotation: cgmath::Deg<f32>,
        transform: cgmath::Vector3<f32>,
        color: [f32; 3],
    ) -> Self {
        let model_transform: cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Basis3<f32>> =
            cgmath::Decomposed {
                scale: 1.0,
                rot: cgmath::Rotation3::from_angle_z(rotation),
                disp: transform,
            };

        Self {
            model: model_transform.into(),
            color: [color[0], color[1], color[2], 1.0],
        }
    }

    pub fn update_rotation(&mut self, rotation: cgmath::Deg<f32>) {
        use cgmath::Zero;
        let new_rotation: cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Basis3<f32>> =
            cgmath::Decomposed {
                scale: 1.0,
                rot: cgmath::Rotation3::from_angle_z(rotation),
                disp: cgmath::Vector3::zero(),
            };
        self.model = self.model * cgmath::Matrix4::from(new_rotation);
    }
}

impl UniformBufferObject {
    pub fn new(swapchain: &Swapchain) -> UniformBufferObject {
        UniformBufferObject {
            view: Matrix4::look_at(
                Point3::new(0.0, 15.0, 10.5),
                Point3::new(0.0, 0.0, 1.0),
                Vector3::new(0.0, 0.0, 1.0),
            ),
            proj: {
                let proj = cgmath::perspective(
                    Deg(45.0),
                    swapchain.extent.width as f32 / swapchain.extent.height as f32,
                    0.1,
                    100.0,
                );
                examples::OPENGL_TO_VULKAN_MATRIX * proj
            },
        }
    }
}

impl Light {
    pub fn new(
        pos: cgmath::Point3<f32>,
        aspect: f32,
        color: [f32; 3],
        ambient: f32,
        specular: f32,
    ) -> Light {
        let view = Matrix4::look_at(pos, Point3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 1.0));
        let projection = cgmath::perspective(Deg(45.0), aspect, 0.1, 15.0);

        Light {
            projection: examples::OPENGL_TO_VULKAN_MATRIX * projection * view,
            pos,
            color,
            ambient,
            specular,
        }
    }
}

pub struct Pipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,

    pub texture: Image,
    pub depth_image: Image,
    pub sampler: vk::Sampler,
    pub uniform_buffer: Buffer,
    pub uniform_transform: UniformBufferObject,

    pub light_buffer: Buffer,
    pub light: Light,

    pub descriptor_layout: vk::DescriptorSetLayout,
    pub descriptor_set: vk::DescriptorSet,
    pub descriptor_pool: vk::DescriptorPool,

    context: Arc<Context>,
}

impl Pipeline {
    //Creates a new pipeline
    pub fn create_pipeline(
        swapchain: &Swapchain,
        renderpass: vk::RenderPass,
        vulkan: &VkInstance,
    ) -> Pipeline {
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
                    format: vk::Format::R32G32_SFLOAT,
                    offset: offset_of!(Vertex, tex_cord) as u32,
                },
                vk::VertexInputAttributeDescription {
                    binding: 0,
                    location: 2,
                    format: vk::Format::R32G32_SFLOAT,
                    offset: offset_of!(Vertex, normal) as u32,
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

        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]);

        let vertex_shader = load_shader(&Path::new("src/bin/lights/shaders/mesh.vert.spv"));
        let frag_shader = load_shader(&Path::new("src/bin/lights/shaders/mesh.frag.spv"));

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

        let texture = create_texture(&Path::new("assets/texture.jpg"), &vulkan);

        let depth_image = examples::create_depth_resources(&swapchain, &vulkan);

        let sampler_create_info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            max_lod: 1.0,
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

        let uniform_data = UniformBufferObject::new(&swapchain);

        let light_data = Light::new(
            cgmath::Point3::new(0.0, 3.0, 3.0),
            0.5,
            [1.0, 1.5, 1.0],
            0.5,
            0.5,
        );

        let uniform_buffer = Buffer::new_mapped_basic(
            std::mem::size_of_val(&uniform_data) as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk_mem::MemoryUsage::CpuOnly,
            vulkan.context(),
        );

        let light_buffer = Buffer::new_mapped_basic(
            std::mem::size_of_val(&light_data) as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk_mem::MemoryUsage::CpuOnly,
            vulkan.context(),
        );

        uniform_buffer.upload_to_buffer(&[uniform_data], 0);
        light_buffer.upload_to_buffer(&[light_data], 0);

        let descriptor_binding = vec![
            vk::DescriptorSetLayoutBinding {
                // transform uniform
                binding: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                p_immutable_samplers: ptr::null(),
            },
            vk::DescriptorSetLayoutBinding {
                // transform uniform
                binding: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                p_immutable_samplers: ptr::null(),
            },
            vk::DescriptorSetLayoutBinding {
                // sampler uniform
                binding: 2,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                p_immutable_samplers: ptr::null(),
            },
        ];

        let (descriptor_layout, descriptor_set, descriptor_pool) =
            vulkan.context.create_descriptor(
                swapchain.image_views.len() as u32,
                descriptor_binding,
                &mut vec![
                    vk::WriteDescriptorSet {
                        // transform uniform
                        dst_binding: 0,
                        dst_array_element: 0,
                        descriptor_count: 1,
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        p_buffer_info: [vk::DescriptorBufferInfo {
                            buffer: uniform_buffer.buffer,
                            offset: 0,
                            range: std::mem::size_of_val(&uniform_data) as u64,
                        }]
                        .as_ptr(),
                        ..Default::default()
                    },
                    vk::WriteDescriptorSet {
                        // light uniform
                        dst_binding: 1,
                        dst_array_element: 0,
                        descriptor_count: 1,
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        p_buffer_info: [vk::DescriptorBufferInfo {
                            buffer: light_buffer.buffer,
                            offset: 0,
                            range: std::mem::size_of_val(&light_data) as u64,
                        }]
                        .as_ptr(),
                        ..Default::default()
                    },
                    vk::WriteDescriptorSet {
                        // sampler uniform
                        dst_binding: 2,
                        dst_array_element: 0,
                        descriptor_count: 1,
                        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        p_image_info: [vk::DescriptorImageInfo {
                            sampler: sampler,
                            image_view: texture.view(),
                            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                        }]
                        .as_ptr(),
                        ..Default::default()
                    },
                ],
            );

        let layout = &[descriptor_layout];

        let push_constant_range = vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .size(mem::size_of::<PushConstantModel>() as u32)
            .build();

        let layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(layout)
            .push_constant_ranges(&[push_constant_range])
            .build();

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
            context: vulkan.context(),
            uniform_buffer,
            light_buffer,
            light: light_data,
            uniform_transform: uniform_data,

            descriptor_set,
            descriptor_layout,
            descriptor_pool,
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
                .destroy_image_view(self.texture.view(), None);
            self.context
                .device
                .destroy_image_view(self.depth_image.view(), None);

            self.context
                .device
                .destroy_descriptor_set_layout(self.descriptor_layout, None);
            self.context
                .device
                .destroy_descriptor_pool(self.descriptor_pool, None);
        }
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
        mip_levels: 1,
        array_layers: 1,
        samples: vk::SampleCountFlags::TYPE_1,
        tiling: vk::ImageTiling::OPTIMAL,
        usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };

    let mut image = Image::create_image(
        image_create_info,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vulkan.context(),
    );

    vulkan.transition_image_layout(
        image.image,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        1,
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

    vulkan.transition_image_layout(
        image.image,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        1,
    );

    image.attach_view(vk::ImageViewCreateInfo {
        s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
        view_type: vk::ImageViewType::TYPE_2D,
        format: vk::Format::R8G8B8A8_UNORM,
        image: image.image,
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

    image
}


pub fn create_render_pass(swapchain: &Swapchain, vulkan: &VkInstance) -> vk::RenderPass {
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

    let depth_format = vulkan.find_depth_format(
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
