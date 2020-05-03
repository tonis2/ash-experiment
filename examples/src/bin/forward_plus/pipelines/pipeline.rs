use vulkan::{
    offset_of, prelude::*, utilities::as_byte_slice, Buffer, Descriptor, DescriptorSet,
    Framebuffer, Image, Pipeline, Renderpass, Shader, Swapchain, VkThread,
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
    pub depth_pass: Renderpass,

    //Forward renderer
    pub forward_descriptor: Descriptor,
    pub forward_pass: Renderpass,

    pub light_buffer: Buffer,
    pub material_buffer: Buffer,

    pub pipelines: Pipeline,

    _empty_image: Image,
}

pub fn new(scene: &Scene, swapchain: &Swapchain, vulkan: &VkThread) {
    //Pipeline stuff
    let viewports = vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: swapchain.width() as f32,
        height: swapchain.height() as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    };

    let scissors = vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: swapchain.extent,
    };

    let noop_stencil_state = vk::StencilOpState {
        fail_op: vk::StencilOp::KEEP,
        pass_op: vk::StencilOp::KEEP,
        depth_fail_op: vk::StencilOp::KEEP,
        compare_op: vk::CompareOp::ALWAYS,
        ..Default::default()
    };

    //Create camera buffer
    let camera = Camera::new(cgmath::Point3::new(0.0, 0.0, 0.0), 15.0, 1.3);

    let camera_buffer = Buffer::new_mapped_basic(
        mem::size_of::<CameraRaw>() as u64,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        vk_mem::MemoryUsage::CpuOnly,
        vulkan.context(),
    );

    camera_buffer.upload_to_buffer(&[camera.raw()], 0);

    //Create light buffers
    let light_buffer = vulkan.create_gpu_buffer(
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        &scene
            .get_lights()
            .map_or(vec![Light::default()], |lights| lights.clone()),
    );

    //Pipeline bindings
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

    //Bind scene textures
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

    //Attributes

    //Forward renderer
    let forward_attributes = [
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

    //Depth attributes
    let depth_attributes = [vk::VertexInputAttributeDescription {
        binding: 0,
        location: 0,
        format: vk::Format::R32G32B32_SFLOAT,
        offset: offset_of!(Vertex, position) as u32,
    }];

    //Descriptors

    let depth_descriptor = Descriptor::new(
        vec![DescriptorSet {
            bind_index: 0,
            flag: vk::ShaderStageFlags::VERTEX,
            bind_type: vk::DescriptorType::UNIFORM_BUFFER,
            buffer_info: Some(vec![vk::DescriptorBufferInfo {
                buffer: camera_buffer.buffer,
                offset: mem::size_of::<CameraRaw>() as u64,
                range: camera_buffer.size,
            }]),
            ..Default::default()
        }],
        vulkan.context(),
    );

    let forward_descriptor = Descriptor::new(
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
        vulkan.context(),
    );

    //Renderpasses

    let forward_pass = Renderpass::new(
        vk::RenderPassCreateInfo::builder()
            .subpasses(&[vk::SubpassDescription::builder()
                .color_attachments(&[vk::AttachmentReference {
                    attachment: 0,
                    layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                }])
                .build()])
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
            .attachments(&[vk::AttachmentDescription {
                flags: vk::AttachmentDescriptionFlags::empty(),
                format: swapchain.format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            }])
            .build(),
        vulkan.context(),
    );

    let depth_image = examples::create_depth_resources(&swapchain, vulkan.context());
    let depth_pass = Renderpass::new(
        vk::RenderPassCreateInfo::builder()
            .attachments(&[vk::AttachmentDescription {
                flags: vk::AttachmentDescriptionFlags::empty(),
                format: depth_image.format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL,
            }])
            .subpasses(&[vk::SubpassDescription::builder()
                .depth_stencil_attachment(&vk::AttachmentReference {
                    attachment: 0,
                    layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                })
                .build()])
            .dependencies(&[
                vk::SubpassDependency {
                    src_subpass: vk::SUBPASS_EXTERNAL,
                    dst_subpass: 0,
                    src_stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER,
                    dst_stage_mask: vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                    src_access_mask: vk::AccessFlags::SHADER_READ,
                    dst_access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                    dependency_flags: vk::DependencyFlags::BY_REGION,
                },
                vk::SubpassDependency {
                    src_subpass: 0,
                    dst_subpass: vk::SUBPASS_EXTERNAL,
                    src_stage_mask: vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                    dst_stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER,
                    src_access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                    dst_access_mask: vk::AccessFlags::SHADER_READ,
                    dependency_flags: vk::DependencyFlags::BY_REGION,
                },
            ])
            .build(),
        vulkan.context(),
    );

    //Shaders
    let shader_name = CString::new("main").unwrap();
    let depth_shader = Shader::new(
        &Path::new("src/bin/forward_plus/shaders/depth.vert.spv"),
        vk::ShaderStageFlags::VERTEX,
        &shader_name,
        vulkan.context(),
    );

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

    let forward_shader = vec![
        Shader::new(
            &Path::new("src/bin/forward_plus/shaders/forward.vert.spv"),
            vk::ShaderStageFlags::VERTEX,
            &shader_name,
            vulkan.context(),
        )
        .info(),
        Shader::new(
            &Path::new("src/bin/forward_plus/shaders/forward.frag.spv"),
            vk::ShaderStageFlags::FRAGMENT,
            &shader_name,
            vulkan.context(),
        )
        .use_specialization(specialization_info)
        .info(),
    ];

    //Create pipelines

    let mut pipelines = Pipeline::new(vulkan.context());

    //Depth layout
    pipelines.add_layout(
        vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&[depth_descriptor.layout])
            .push_constant_ranges(&[vk::PushConstantRange {
                stage_flags: vk::ShaderStageFlags::VERTEX,
                size: mem::size_of::<PushTransform>() as u32,
                offset: 0,
            }])
            .build(),
    );

    //Forward layout
    pipelines.add_layout(
        vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&[forward_descriptor.layout])
            .push_constant_ranges(&[vk::PushConstantRange {
                stage_flags: vk::ShaderStageFlags::VERTEX,
                size: mem::size_of::<PushTransform>() as u32,
                offset: 0,
            }])
            .build(),
    );

    //Pipeline base
    let mut pipeline_description = vk::GraphicsPipelineCreateInfo::builder()
        .stages(&[depth_shader.info()])
        .vertex_input_state(
            &vk::PipelineVertexInputStateCreateInfo::builder()
                .vertex_binding_descriptions(&[vk::VertexInputBindingDescription {
                    binding: 0,
                    stride: mem::size_of::<Vertex>() as u32,
                    input_rate: vk::VertexInputRate::VERTEX,
                }])
                .vertex_attribute_descriptions(&depth_attributes)
                .build(),
        )
        .input_assembly_state(&vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            ..Default::default()
        })
        .viewport_state(
            &vk::PipelineViewportStateCreateInfo::builder()
                .scissors(&[scissors])
                .viewports(&[viewports]),
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
            depth_compare_op: vk::CompareOp::LESS,
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
            &vk::PipelineDynamicStateCreateInfo::builder()
                .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]),
        )
        .layout(pipelines.layout(0))
        .render_pass(depth_pass.pass())
        .build();

    //Build depth pipeline
    pipelines.add_pipeline(pipeline_description);

    //Create forward pipeline
    pipeline_description.p_stages = forward_shader.as_ptr();
    pipeline_description.stage_count = forward_shader.len() as u32;
    pipeline_description.p_vertex_input_state = &vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(&[vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }])
        .vertex_attribute_descriptions(&forward_attributes)
        .build();
    pipeline_description.layout = pipelines.layout(1);
    pipeline_description.render_pass = forward_pass.pass();
    pipelines.add_pipeline(pipeline_description);
}
