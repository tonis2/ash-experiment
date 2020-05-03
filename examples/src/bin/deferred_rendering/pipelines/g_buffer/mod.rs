mod helpers;

use std::{default::Default, ffi::CString, mem, path::Path};
use vulkan::{
    offset_of,
    prelude::*,
    utilities::{as_byte_slice, Shader},
    Buffer, Descriptor, DescriptorSet, Framebuffer, Image, Pipeline, Renderpass, Swapchain,
    VkThread,
};

use super::definitions::{PushTransform, SpecializationData};
use examples::utils::{
    gltf_importer::{MaterialRaw, Scene, Vertex},
    Camera, CameraRaw,
};

pub struct Gbuffer {
    pub pipeline_descriptor: Descriptor,

    _empty_image: Image,

    pub depth: Image,
    pub color: Image,
    pub position: Image,
    pub normal: Image,

    pub uniform_buffer: Buffer,
    pub material_buffer: Buffer,

    pub camera: Camera,
    pub renderpass: Renderpass,
    pub framebuffers: Vec<Framebuffer>,
    pub pipeline: Pipeline,
}

impl Gbuffer {
    //Creates a new g_buffer
    pub fn build(scene: &Scene, swapchain: &Swapchain, vulkan: &VkThread) -> Gbuffer {
        let context = vulkan.context();
        //Create buffer data
        let camera = Camera::new(cgmath::Point3::new(0.0, 0.0, 0.0), 15.0, 1.3);

        let uniform_buffer = Buffer::new_mapped_basic(
            mem::size_of::<CameraRaw>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk_mem::MemoryUsage::CpuOnly,
            context.clone(),
        );

        uniform_buffer.upload_to_buffer(&[camera.raw()], 0);

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

        //Create empty placeholder texture for shader
        let context = vulkan.context();

        let (width, height) = (swapchain.extent.width, swapchain.extent.height);

        //Create the textures that G_Buffer saves to
        let color = helpers::create_image(vk::Format::R8G8B8A8_UNORM, width, height, &vulkan);
        let normal = helpers::create_image(vk::Format::R16G16B16A16_SFLOAT, width, height, &vulkan);
        let position =
            helpers::create_image(vk::Format::R16G16B16A16_SFLOAT, width, height, &vulkan);
        let depth = examples::create_depth_resources(&swapchain, context.clone());

        //Empty texture, so shader wont crash when there is 0 materials
        let empty_image = examples::create_empty_image(&vulkan);
        let texture_data: Vec<vk::DescriptorImageInfo> = {
            if scene.textures.len() > 0 {
                scene
                    .textures
                    .iter()
                    .map(|texture| vk::DescriptorImageInfo {
                        sampler: texture.sampler(),
                        image_view: texture.view(),
                        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    })
                    .collect()
            } else {
                vec![vk::DescriptorImageInfo {
                    sampler: empty_image.sampler(),
                    image_view: empty_image.view(),
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                }]
            }
        };

        //Fill with empty texture when scene has no textures.
        let texture_count: u32 = match scene.textures.len() {
            0 => 1,
            _ => scene.textures.len() as u32,
        };

        let pipeline_descriptor = Descriptor::new(
            vec![
                DescriptorSet {
                    bind_index: 0,
                    flag: vk::ShaderStageFlags::VERTEX,
                    bind_type: vk::DescriptorType::UNIFORM_BUFFER,
                    buffer_info: Some(vec![vk::DescriptorBufferInfo {
                        buffer: uniform_buffer.buffer,
                        offset: 0,
                        range: uniform_buffer.size,
                    }]),
                    ..Default::default()
                },
                DescriptorSet {
                    bind_index: 1,
                    flag: vk::ShaderStageFlags::FRAGMENT,
                    bind_type: vk::DescriptorType::UNIFORM_BUFFER,
                    buffer_info: Some(material_bindings),
                    ..Default::default()
                },
                DescriptorSet {
                    bind_index: 2,
                    flag: vk::ShaderStageFlags::FRAGMENT,
                    bind_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    image_info: Some(texture_data),
                    count: texture_count,
                    ..Default::default()
                },
            ],
            context.clone(),
        );

        let noop_stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            ..Default::default()
        };

        let specialization_data = SpecializationData {
            materials_amount: scene.materials.len() as u32,
            textures_amount: scene.textures.len() as u32,
            lights_amount: scene.lights.len() as u32,
        };

        let specialization_info = unsafe {
            vk::SpecializationInfo::builder()
                .map_entries(&specialization_data.specialization_map_entries())
                .data(as_byte_slice(&specialization_data))
                .build()
        };

        //RENDERPASS
        let images: Vec<&Image> = vec![&color, &normal, &position, &depth];
        let mut attachments: Vec<vk::AttachmentDescription> = Vec::new();
        let mut attachment_references: Vec<vk::AttachmentReference> = Vec::new();
        let mut blend_attachments: Vec<vk::PipelineColorBlendAttachmentState> = Vec::new();

        images.iter().enumerate().for_each(|(index, img)| {
            if index < 3 {
                attachments.push(vk::AttachmentDescription {
                    flags: vk::AttachmentDescriptionFlags::empty(),
                    format: img.format,
                    samples: vk::SampleCountFlags::TYPE_1,
                    load_op: vk::AttachmentLoadOp::CLEAR,
                    store_op: vk::AttachmentStoreOp::STORE,
                    stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                    stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                    initial_layout: vk::ImageLayout::UNDEFINED,
                    final_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                });
                attachment_references.push(vk::AttachmentReference {
                    attachment: index as u32,
                    layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                });
                blend_attachments.push(vk::PipelineColorBlendAttachmentState {
                    blend_enable: 0,
                    src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
                    dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
                    color_blend_op: vk::BlendOp::ADD,
                    src_alpha_blend_factor: vk::BlendFactor::ZERO,
                    dst_alpha_blend_factor: vk::BlendFactor::ZERO,
                    alpha_blend_op: vk::BlendOp::ADD,
                    color_write_mask: vk::ColorComponentFlags::all(),
                });
            } else {
                //Depth
                attachments.push(vk::AttachmentDescription {
                    flags: vk::AttachmentDescriptionFlags::empty(),
                    format: img.format,
                    samples: vk::SampleCountFlags::TYPE_1,
                    load_op: vk::AttachmentLoadOp::CLEAR,
                    store_op: vk::AttachmentStoreOp::STORE,
                    stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                    stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                    initial_layout: vk::ImageLayout::UNDEFINED,
                    final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                });
                attachment_references.push(vk::AttachmentReference {
                    attachment: index as u32,
                    layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                });
            }
        });

        let renderpass = Renderpass::new(
            vk::RenderPassCreateInfo::builder()
                .attachments(&attachments)
                .subpasses(&[vk::SubpassDescription::builder()
                    .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                    .color_attachments(&attachment_references[0..3])
                    .depth_stencil_attachment(&attachment_references[3..4][0])
                    .build()])
                .dependencies(&[
                    vk::SubpassDependency {
                        src_subpass: vk::SUBPASS_EXTERNAL,
                        dst_subpass: 0,
                        src_stage_mask: vk::PipelineStageFlags::ALL_GRAPHICS,
                        dst_stage_mask: vk::PipelineStageFlags::ALL_GRAPHICS,
                        src_access_mask: vk::AccessFlags::MEMORY_READ,
                        dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                            | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                        dependency_flags: vk::DependencyFlags::BY_REGION,
                    },
                    vk::SubpassDependency {
                        src_subpass: 0,
                        dst_subpass: vk::SUBPASS_EXTERNAL,
                        src_stage_mask: vk::PipelineStageFlags::ALL_GRAPHICS,
                        dst_stage_mask: vk::PipelineStageFlags::ALL_GRAPHICS,
                        src_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                            | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                        dst_access_mask: vk::AccessFlags::MEMORY_READ,
                        dependency_flags: vk::DependencyFlags::BY_REGION,
                    },
                ])
                .build(),
            vulkan.context(),
        );

        //FRAMEBUFFERS
        let framebuffers: Vec<Framebuffer> = swapchain
            .image_views
            .iter()
            .map(|_image| {
                Framebuffer::new(
                    vk::FramebufferCreateInfo::builder()
                        .layers(1)
                        .render_pass(renderpass.pass())
                        .attachments(&[color.view(), normal.view(), position.view(), depth.view()])
                        .width(swapchain.width())
                        .height(swapchain.height())
                        .build(),
                    vulkan.context(),
                )
            })
            .collect();

        let attributes = [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, position) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, color) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, tangents) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 3,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, normal) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 4,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Vertex, uv) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 5,
                format: vk::Format::R32_SINT,
                offset: offset_of!(Vertex, material_id) as u32,
            },
        ];

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain.extent,
        }];

        let shader_name = CString::new("main").unwrap();
        let mut pipeline = Pipeline::new(vulkan.context());
        pipeline.add_layout(
            vk::PipelineLayoutCreateInfo::builder()
                .set_layouts(&[pipeline_descriptor.layout])
                .push_constant_ranges(&[vk::PushConstantRange {
                    stage_flags: vk::ShaderStageFlags::VERTEX,
                    size: mem::size_of::<PushTransform>() as u32,
                    offset: 0,
                }])
                .build(),
        );

        pipeline.add_pipeline(
            vk::GraphicsPipelineCreateInfo::builder()
                .stages(&[
                    Shader::new(
                        &Path::new("src/bin/deferred_rendering/shaders/gbuffer.vert.spv"),
                        vk::ShaderStageFlags::VERTEX,
                        &shader_name,
                        context.clone(),
                    )
                    .info(),
                    Shader::new(
                        &Path::new("src/bin/deferred_rendering/shaders/gbuffer.frag.spv"),
                        vk::ShaderStageFlags::FRAGMENT,
                        &shader_name,
                        context.clone(),
                    )
                    .use_specialization(specialization_info)
                    .info(),
                ])
                .vertex_input_state(
                    &vk::PipelineVertexInputStateCreateInfo::builder()
                        .vertex_binding_descriptions(&[vk::VertexInputBindingDescription {
                            binding: 0,
                            stride: mem::size_of::<Vertex>() as u32,
                            input_rate: vk::VertexInputRate::VERTEX,
                        }])
                        .vertex_attribute_descriptions(&attributes)
                        .build(),
                )
                .input_assembly_state(&vk::PipelineInputAssemblyStateCreateInfo {
                    topology: vk::PrimitiveTopology::TRIANGLE_LIST,
                    ..Default::default()
                })
                .viewport_state(
                    &vk::PipelineViewportStateCreateInfo::builder()
                        .scissors(&scissors)
                        .viewports(&viewports),
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
                        .attachments(&blend_attachments),
                )
                .dynamic_state(
                    &vk::PipelineDynamicStateCreateInfo::builder()
                        .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]),
                )
                .render_pass(renderpass.pass())
                .layout(pipeline.layout(0)),
        );

        Self {
            pipeline,
            pipeline_descriptor,
            color,
            position,
            normal,
            depth,
            _empty_image: empty_image,
            uniform_buffer,
            material_buffer,
            camera,
            renderpass,
            framebuffers,
        }
    }

    pub fn get_buffer_images(&self) -> Vec<&Image> {
        vec![&self.color, &self.position, &self.normal]
    }
}
