use vulkan::{
    modules::swapchain::Swapchain,
    offset_of,
    utilities::{tools::load_shader, Buffer},
    VkInstance,
};

use ash::version::DeviceV1_0;
use std::mem::{self, align_of};

use ash::vk;
use std::default::Default;
use std::ffi::CString;
use std::path::Path;

#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 4],
}

pub fn create_index_buffer(indices: &Vec<u16>, vulkan: &VkInstance) -> Buffer {
    let size = std::mem::size_of_val(&indices) as vk::DeviceSize * indices.len() as u64;

    let mut staging_buffer = vulkan.create_buffer(
        vk::BufferCreateInfo {
            size,
            usage: vk::BufferUsageFlags::TRANSFER_SRC,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        },
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );

    let buffer = vulkan.create_buffer(
        vk::BufferCreateInfo {
            size,
            usage: vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        },
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );
    staging_buffer.copy_to_buffer_dynamic(align_of::<u32>() as u64, &indices, &vulkan);
    vulkan.copy_buffer(staging_buffer, buffer);
    
    staging_buffer.destroy(&vulkan);
    buffer
}

pub fn create_vertex_buffer(vertices: &[Vertex], vulkan: &VkInstance) -> Buffer {
    let mut staging_buffer = vulkan.create_buffer(
        vk::BufferCreateInfo {
            size: std::mem::size_of_val(vertices) as vk::DeviceSize,
            usage: vk::BufferUsageFlags::TRANSFER_SRC,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        },
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );

    staging_buffer.copy_to_buffer_dynamic(align_of::<Vertex>() as u64, &vertices, &vulkan);

    let buffer = vulkan.create_buffer(
        vk::BufferCreateInfo {
            size: std::mem::size_of_val(vertices) as vk::DeviceSize,
            usage: vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        },
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );

    vulkan.copy_buffer(staging_buffer, buffer);

    staging_buffer.destroy(&vulkan);

    buffer
}

//Creates a new pipeline
pub fn create_pipeline(
    swapchain: &Swapchain,
    renderpass: vk::RenderPass,
    vulkan: &vulkan::VkInstance,
) -> (vk::Pipeline, vk::PipelineLayout) {
    let vertex_binding = vec![vk::VertexInputBindingDescription {
        binding: 0,
        stride: mem::size_of::<Vertex>() as u32,
        input_rate: vk::VertexInputRate::VERTEX,
    }];
    let vertex_attributes = vec![
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

    let layout_create_info = vk::PipelineLayoutCreateInfo::default();

    let pipeline_layout = unsafe {
        vulkan
            .device
            .create_pipeline_layout(&layout_create_info, None)
            .unwrap()
    };

    let vertex_shader = load_shader(&Path::new(
        "examples/triangle/shaders/spv/triangle.vert.spv",
    ));
    let frag_shader = load_shader(&Path::new(
        "examples/triangle/shaders/spv/triangle.frag.spv",
    ));

    let vertex_shader_module = unsafe {
        vulkan
            .device
            .create_shader_module(
                &vk::ShaderModuleCreateInfo::builder().code(&vertex_shader),
                None,
            )
            .expect("Vertex shader module error")
    };

    let fragment_shader_module = unsafe {
        vulkan
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
        vulkan
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
        vulkan
            .device
            .destroy_shader_module(vertex_shader_module, None);
        vulkan
            .device
            .destroy_shader_module(fragment_shader_module, None);
    }
    (pipeline[0], pipeline_layout)
}
