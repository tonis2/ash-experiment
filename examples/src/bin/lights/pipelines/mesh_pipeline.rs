use vulkan::{
    modules::swapchain::Swapchain,
    offset_of,
    prelude::*,
    utilities::{tools::load_shader, Image},
    Buffer, Context, VkInstance,
};

use super::shadowmap_pipeline;

use cgmath::{Deg, Matrix4, Point3, Vector3};

use std::{default::Default, ffi::CString, mem, path::Path, ptr, sync::Arc};

#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: [f32; 3],
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
        transform: cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Basis3<f32>>,
        color: [f32; 3],
    ) -> Self {
        Self {
            model: transform.into(),
            color: [color[0], color[1], color[2], 1.0],
        }
    }

    pub fn update_transform(
        &mut self,
        transform: cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Basis3<f32>>,
    ) {
        self.model = self.model * cgmath::Matrix4::from(transform);
    }
}

impl UniformBufferObject {
    pub fn new(swapchain: &Swapchain) -> UniformBufferObject {
        UniformBufferObject {
            view: Matrix4::look_at(
                Point3::new(0.0, -15.0, 8.0),
                Point3::new(0.0, -5.0, 1.0),
                Vector3::new(0.0, 3.0, 1.0),
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
    pub depth_image: Image,
    pub uniform_buffer: Buffer,
    pub uniform_transform: UniformBufferObject,

    pub light_buffer: Buffer,
    pub light: Light,

    pub renderpass: vk::RenderPass,

    pub descriptor_layout: vk::DescriptorSetLayout,
    pub descriptor_set: vk::DescriptorSet,
    pub descriptor_pool: vk::DescriptorPool,

    pub shadow_map: shadowmap_pipeline::Pipeline,

    context: Arc<Context>,
}

impl Pipeline {
    pub fn update_light(&mut self, light: Light) {
        self.light_buffer.upload_to_buffer(&[light], 0);
    }
    //Creates a new pipeline
    pub fn new(swapchain: &Swapchain, vulkan: &VkInstance) -> Pipeline {
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

        let depth_image = examples::create_depth_resources(&swapchain, &vulkan);

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

        let shadow_map = shadowmap_pipeline::Pipeline::new(&swapchain, vulkan.context.clone());

        let (descriptor_layout, descriptor_set, descriptor_pool) =
            vulkan.context.create_descriptor(
                swapchain.image_views.len() as u32,
                vec![
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
                        // Shadow image
                        binding: 2,
                        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        descriptor_count: 1,
                        stage_flags: vk::ShaderStageFlags::FRAGMENT,
                        p_immutable_samplers: ptr::null(),
                    },
                ],
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
                        // shadow uniform
                        dst_binding: 2,
                        dst_array_element: 0,
                        descriptor_count: 1,
                        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        p_image_info: [vk::DescriptorImageInfo {
                            sampler: shadow_map.sampler,
                            image_view: shadow_map.image.view(),
                            image_layout: vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL,
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

        //Create pipeline stuff
        let pipeline_layout = unsafe {
            vulkan
                .device()
                .create_pipeline_layout(
                    &vk::PipelineLayoutCreateInfo::builder()
                        .set_layouts(layout)
                        .push_constant_ranges(&[push_constant_range])
                        .build(),
                    None,
                )
                .unwrap()
        };
        let renderpass = create_render_pass(&swapchain, &vulkan);
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
            depth_image,
            context: vulkan.context(),
            uniform_buffer,
            light_buffer,
            light: light_data,
            uniform_transform: uniform_data,
            renderpass,
            shadow_map,
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

            self.context
                .device
                .destroy_descriptor_set_layout(self.descriptor_layout, None);
            self.context
                .device
                .destroy_descriptor_pool(self.descriptor_pool, None);

            self.context
                .device
                .destroy_render_pass(self.renderpass, None);
        }
    }
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
