use cgmath::{Deg, Matrix4, Point3, Vector3};
use std::default::Default;
use std::ffi::CString;
use std::mem;
use std::path::Path;

use vulkan::{
    offset_of, prelude::*, Buffer, Descriptor, DescriptorSet, Pipeline, Renderpass, Shader,
    Swapchain, VkThread,
};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct UniformBufferObject {
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 3],
}

pub struct Pipe {
    pub pipeline: Pipeline,
    pub renderpass: Renderpass,
    pub pipeline_descriptor: Descriptor,
    pub uniform_buffer: Buffer,
    pub uniform_transform: UniformBufferObject,
}

impl Pipe {
    //Creates a new pipeline
    pub fn new(swapchain: &Swapchain, vulkan: &VkThread) -> Self {
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

        let noop_stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            ..Default::default()
        };

        //Create uniform buffer

        let uniform_data = create_uniform_data(&swapchain);

        let uniform_buffer =
            vulkan.create_gpu_buffer(vk::BufferUsageFlags::UNIFORM_BUFFER, &[uniform_data]);

        let pipeline_descriptor = Descriptor::new(
            vec![DescriptorSet {
                bind_index: 0,
                flag: vk::ShaderStageFlags::VERTEX,
                bind_type: vk::DescriptorType::UNIFORM_BUFFER,
                buffer_info: Some(vec![vk::DescriptorBufferInfo {
                    buffer: uniform_buffer.buffer,
                    offset: 0,
                    range: uniform_buffer.size,
                }]),
                ..Default::default()
            }],
            vulkan.context(),
        );

        let renderpass = Renderpass::new(
            vk::RenderPassCreateInfo::builder()
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
                .subpasses(&[vk::SubpassDescription::builder()
                    .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
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
                .build(),
            vulkan.context(),
        );

        let shader_name = CString::new("main").unwrap();
        let pipeline = Pipeline::new(
            vk::PipelineLayoutCreateInfo::builder()
                .set_layouts(&[pipeline_descriptor.layout])
                .build(),
            vk::GraphicsPipelineCreateInfo::builder()
                .stages(&[
                    Shader::new(
                        &Path::new("src/bin/triangle/shaders/triangle.vert.spv"),
                        vk::ShaderStageFlags::VERTEX,
                        &shader_name,
                        vulkan.context(),
                    )
                    .info(),
                    Shader::new(
                        &Path::new("src/bin/triangle/shaders/triangle.frag.spv"),
                        vk::ShaderStageFlags::FRAGMENT,
                        &shader_name,
                        vulkan.context(),
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
                                offset: offset_of!(Vertex, pos) as u32,
                            },
                            vk::VertexInputAttributeDescription {
                                binding: 0,
                                location: 1,
                                format: vk::Format::R32G32B32_SFLOAT,
                                offset: offset_of!(Vertex, color) as u32,
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
                .render_pass(renderpass.pass()),
            vulkan.context(),
        );

        Self {
            pipeline,
            renderpass,
            pipeline_descriptor,
            uniform_buffer,
            uniform_transform: uniform_data,
        }
    }
}

pub fn create_uniform_data(swapchain: &Swapchain) -> UniformBufferObject {
    UniformBufferObject {
        view: Matrix4::look_at(
            Point3::new(2.0, 2.0, 2.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        ),
        proj: {
            let proj = cgmath::perspective(
                Deg(45.0),
                swapchain.extent.width as f32 / swapchain.extent.height as f32,
                0.1,
                10.0,
            );
            examples::OPENGL_TO_VULKAN_MATRIX * proj
        },
    }
}
