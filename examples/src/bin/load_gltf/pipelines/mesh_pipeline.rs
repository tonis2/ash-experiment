use vulkan::{
    modules::swapchain::Swapchain,
    offset_of,
    prelude::*,
    utilities::{as_byte_slice, Shader},
    Buffer, Context, Descriptor, Image,
};

use super::{Camera, PushTransform, SpecializationData};
use examples::gltf_importer::{self, MaterialRaw, Vertex};
use std::{default::Default, ffi::CString, mem, path::Path, ptr, sync::Arc};

pub struct Pipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub pipeline_descriptor: Descriptor,

    pub depth_image: Image,
    pub uniform_buffer: Buffer,
    pub uniform_transform: Camera,
    pub renderpass: vk::RenderPass,

    context: Arc<Context>,
}

impl Pipeline {
    //Creates a new pipeline
    pub fn build_for(
        scene: &gltf_importer::Scene,
        swapchain: &Swapchain,
        context: Arc<Context>,
    ) -> Pipeline {
        //Create buffer data

        let camera = Camera::new(800.0 / 600.0, cgmath::Point3::new(0.0, 5.0, 15.0));
        let depth_image = examples::create_depth_resources(&swapchain, context.clone());

        let uniform_buffer = Buffer::new_mapped_basic(
            mem::size_of::<Camera>() as vk::DeviceSize,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk_mem::MemoryUsage::CpuOnly,
            context.clone(),
        );
        uniform_buffer.upload_to_buffer(&[camera], 0);

        let material_buffer = Buffer::new_mapped_basic(
            context.get_ubo_alignment::<MaterialRaw>() as vk::DeviceSize,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk_mem::MemoryUsage::CpuOnly,
            context.clone(),
        );

        material_buffer.upload_to_buffer(&scene.get_raw_materials()[..], 0);

        let texture_data: Vec<vk::DescriptorImageInfo> = scene
            .textures
            .iter()
            .map(|texture| vk::DescriptorImageInfo {
                sampler: texture.sampler(),
                image_view: texture.view(),
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            })
            .collect();

        let pipeline_descriptor = Descriptor::new(
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
                    // Material uniform
                    binding: 1,
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
                    descriptor_count: 1,
                    stage_flags: vk::ShaderStageFlags::FRAGMENT,
                    p_immutable_samplers: ptr::null(),
                },
                vk::DescriptorSetLayoutBinding {
                    // Textures uniform
                    binding: 2,
                    descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: scene.textures.len() as u32,
                    stage_flags: vk::ShaderStageFlags::FRAGMENT,
                    p_immutable_samplers: ptr::null(),
                },
            ],
            vec![
                vk::WriteDescriptorSet {
                    // transform uniform
                    dst_binding: 0,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                    p_buffer_info: [vk::DescriptorBufferInfo {
                        buffer: uniform_buffer.buffer,
                        offset: 0,
                        range: uniform_buffer.size,
                    }]
                    .as_ptr(),
                    ..Default::default()
                },
                vk::WriteDescriptorSet {
                    // Material uniform
                    dst_binding: 1,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
                    p_buffer_info: [vk::DescriptorBufferInfo {
                        buffer: material_buffer.buffer,
                        offset: 0,
                        range: material_buffer.size,
                    }]
                    .as_ptr(),
                    ..Default::default()
                },
                vk::WriteDescriptorSet {
                    // Material uniform
                    dst_binding: 2,
                    dst_array_element: 0,
                    descriptor_count: scene.textures.len() as u32,
                    descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    p_image_info: texture_data.as_ptr(),
                    ..Default::default()
                },
            ],
            context.clone(),
        );

        //Create pipeline stuff
        let pipeline_layout = unsafe {
            context
                .device
                .create_pipeline_layout(
                    &vk::PipelineLayoutCreateInfo::builder()
                        .set_layouts(&[pipeline_descriptor.layout])
                        .push_constant_ranges(&[vk::PushConstantRange {
                            stage_flags: vk::ShaderStageFlags::VERTEX,
                            size: mem::size_of::<PushTransform>() as u32,
                            offset: 0,
                        }])
                        .build(),
                    None,
                )
                .unwrap()
        };

        let noop_stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            ..Default::default()
        };

        let specialization_info = unsafe {
            vk::SpecializationInfo::builder()
                .map_entries(&vec![
                    vk::SpecializationMapEntry {
                        constant_id: 0,
                        offset: 0,
                        size: std::mem::size_of::<u32>(),
                    },
                    vk::SpecializationMapEntry {
                        constant_id: 1,
                        offset: std::mem::size_of::<u32>() as _,
                        size: std::mem::size_of::<u32>(),
                    },
                ])
                .data(&as_byte_slice(&[
                    scene.textures.len() as u32,
                    scene.materials.len() as u32,
                ]))
                .build()
        };

        let shader_name = CString::new("main").unwrap();
        let renderpass = create_render_pass(&swapchain, context.clone());
        let pipeline = unsafe {
            context
                .device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[vk::GraphicsPipelineCreateInfo::builder()
                        .stages(&[
                            Shader::new(
                                &Path::new("src/bin/load_gltf/shaders/model.vert.spv"),
                                vk::ShaderStageFlags::VERTEX,
                                &shader_name,
                                context.clone(),
                            )
                            .info(),
                            Shader::new(
                                &Path::new("src/bin/load_gltf/shaders/model.frag.spv"),
                                vk::ShaderStageFlags::FRAGMENT,
                                &shader_name,
                                context.clone(),
                            )
                            .info(),
                        ])
                        .vertex_input_state(
                            &vk::PipelineVertexInputStateCreateInfo::builder()
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
                                        offset: offset_of!(Vertex, position) as u32,
                                    },
                                    vk::VertexInputAttributeDescription {
                                        binding: 0,
                                        location: 1,
                                        format: vk::Format::R32G32_SFLOAT,
                                        offset: offset_of!(Vertex, color) as u32,
                                    },
                                    vk::VertexInputAttributeDescription {
                                        binding: 0,
                                        location: 2,
                                        format: vk::Format::R32G32_SFLOAT,
                                        offset: offset_of!(Vertex, normal) as u32,
                                    },
                                    vk::VertexInputAttributeDescription {
                                        binding: 0,
                                        location: 3,
                                        format: vk::Format::R32G32_SFLOAT,
                                        offset: offset_of!(Vertex, uv) as u32,
                                    },
                                ])
                                .build(),
                        )
                        .input_assembly_state(&vk::PipelineInputAssemblyStateCreateInfo {
                            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
                            ..Default::default()
                        })
                        .viewport_state(
                            &vk::PipelineViewportStateCreateInfo::builder()
                                .scissors(&[vk::Rect2D {
                                    offset: vk::Offset2D { x: 0, y: 0 },
                                    extent: swapchain.extent,
                                }])
                                .viewports(&[vk::Viewport {
                                    x: 0.0,
                                    y: 0.0,
                                    width: swapchain.extent.width as f32,
                                    height: swapchain.extent.height as f32,
                                    min_depth: 0.0,
                                    max_depth: 1.0,
                                }]),
                        )
                        .rasterization_state(&vk::PipelineRasterizationStateCreateInfo {
                            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
                            line_width: 1.0,
                            polygon_mode: vk::PolygonMode::FILL,
                            ..Default::default()
                        })
                        .multisample_state(&vk::PipelineMultisampleStateCreateInfo {
                            rasterization_samples: vk::SampleCountFlags::TYPE_1,
                            ..Default::default()
                        })
                        .depth_stencil_state(&vk::PipelineDepthStencilStateCreateInfo {
                            depth_test_enable: 1,
                            depth_write_enable: 1,
                            depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
                            front: noop_stencil_state,
                            back: noop_stencil_state,
                            max_depth_bounds: 1.0,
                            ..Default::default()
                        })
                        .color_blend_state(
                            &vk::PipelineColorBlendStateCreateInfo::builder()
                                .logic_op(vk::LogicOp::CLEAR)
                                .attachments(&[vk::PipelineColorBlendAttachmentState {
                                    blend_enable: 0,
                                    src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
                                    dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
                                    color_blend_op: vk::BlendOp::ADD,
                                    src_alpha_blend_factor: vk::BlendFactor::ZERO,
                                    dst_alpha_blend_factor: vk::BlendFactor::ZERO,
                                    alpha_blend_op: vk::BlendOp::ADD,
                                    color_write_mask: vk::ColorComponentFlags::all(),
                                }]),
                        )
                        .dynamic_state(
                            &vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&[
                                vk::DynamicState::VIEWPORT,
                                vk::DynamicState::SCISSOR,
                            ]),
                        )
                        .layout(pipeline_layout)
                        .render_pass(renderpass)
                        .build()],
                    None,
                )
                .expect("Unable to create graphics pipeline")
        }[0];

        Pipeline {
            pipeline: pipeline,
            pipeline_descriptor,
            layout: pipeline_layout,
            depth_image,
            uniform_buffer,
            uniform_transform: camera,
            renderpass,
            context,
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

pub fn create_render_pass(swapchain: &Swapchain, context: Arc<Context>) -> vk::RenderPass {
    let depth_format = context.find_depth_format(
        &[
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ],
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    );

    let subpasses = vk::SubpassDescription::builder()
        .color_attachments(&[vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }])
        .depth_stencil_attachment(&vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        })
        .build();

    let renderpass_create_info = vk::RenderPassCreateInfo::builder()
        .subpasses(&[subpasses])
        .dependencies(&[vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::empty(),
        }])
        .attachments(&[
            vk::AttachmentDescription {
                flags: vk::AttachmentDescriptionFlags::empty(),
                format: swapchain.format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            },
            vk::AttachmentDescription {
                flags: vk::AttachmentDescriptionFlags::empty(),
                format: depth_format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::DONT_CARE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            },
        ])
        .build();

    unsafe {
        context
            .device
            .create_render_pass(&renderpass_create_info, None)
            .expect("Failed to create render pass!")
    }
}
