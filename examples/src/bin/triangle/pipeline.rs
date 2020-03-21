use vulkan::{
    modules::swapchain::Swapchain, offset_of, prelude::*, utilities::tools::load_shader, Context,
};

use std::sync::Arc;

use std::mem;

use std::ffi::CString;
use std::path::Path;

#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 4],
}

//Creates a new pipeline
pub fn create_pipeline(
    swapchain: &Swapchain,
    renderpass: vk::RenderPass,
    context: Arc<Context>,
) -> (vk::Pipeline, vk::PipelineLayout) {
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

    let layout_create_info = vk::PipelineLayoutCreateInfo::default();

    let pipeline_layout = unsafe {
        context
            .device
            .create_pipeline_layout(&layout_create_info, None)
            .unwrap()
    };

    let vertex_shader = load_shader(&Path::new("src/bin/triangle/shaders/triangle.vert.spv"));
    let frag_shader = load_shader(&Path::new("src/bin/triangle/shaders/triangle.frag.spv"));

    let vertex_shader_module = unsafe {
        context
            .device
            .create_shader_module(
                &vk::ShaderModuleCreateInfo::builder().code(&vertex_shader),
                None,
            )
            .expect("Vertex shader module error")
    };

    let fragment_shader_module = unsafe {
        context
            .device
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
        context
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
        context
            .device
            .destroy_shader_module(vertex_shader_module, None);
        context
            .device
            .destroy_shader_module(fragment_shader_module, None);
    }
    (pipeline[0], pipeline_layout)
}
