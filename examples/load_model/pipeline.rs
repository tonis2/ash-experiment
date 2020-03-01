use vulkan::{
    modules::swapchain::Swapchain,
    offset_of,
    utilities::{Shader, VertexDescriptor, DescriptorInfo}, VkInstance,
};

use ash::util::Align;
use ash::version::DeviceV1_0;
use ash::vk;
use cgmath::{Deg, Matrix4, Point3, SquareMatrix, Vector3};
use std::default::Default;
use std::ffi::CString;
use std::mem::{self, align_of};
use std::ptr;

#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct UniformBufferObject {
    model: Matrix4<f32>,
    view: Matrix4<f32>,
    proj: Matrix4<f32>,
}

//Creates a new pipeline

pub fn create_pipeline(
    swapchain: &Swapchain,
    renderpass: vk::RenderPass,
    vulkan: &VkInstance,
) -> (
    vk::Pipeline,
    vk::PipelineLayout,
    VertexDescriptor,
    DescriptorInfo,
) {
    let vertex_descriptor = VertexDescriptor {
        binding_descriptor: vec![vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }],
        attribute_descriptor: vec![
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
        ],
        size: 3 * std::mem::size_of::<Vertex>() as u64,
        align: align_of::<Vertex>() as u64,
    };

    let descriptor_len = vertex_descriptor.attribute_descriptor.len() as u32;
    let binding_len = vertex_descriptor.binding_descriptor.len() as u32;

    unsafe {
        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo {
            vertex_attribute_description_count: descriptor_len,
            p_vertex_attribute_descriptions: vertex_descriptor.attribute_descriptor.as_ptr(),
            vertex_binding_description_count: binding_len,
            p_vertex_binding_descriptions: vertex_descriptor.binding_descriptor.as_ptr(),
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

        let (vertex_shader_module, fragment_shader_module) = Shader {
            vertex_shader: "examples/load_model/shaders/spv/shader-ubo.vert.spv",
            fragment_shader: "examples/load_model/shaders/spv/shader-ubo.frag.spv",
        }
        .load(&vulkan);

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

        let uniform_data = create_uniform_data(&swapchain);

        let buffer_create_info = vk::BufferCreateInfo {
            size: std::mem::size_of_val(&uniform_data) as u64,
            usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let mut uniform_buffer = vulkan.create_buffer(buffer_create_info);

        let uniform_ptr = vulkan
            .device
            .map_memory(
                uniform_buffer.memory,
                0,
                uniform_buffer.memory_requirements.size,
                vk::MemoryMapFlags::empty(),
            )
            .unwrap();

        let mut uniform_aligned_slice = Align::new(
            uniform_ptr,
            align_of::<UniformBufferObject>() as u64,
            uniform_buffer.memory_requirements.size,
        );
        uniform_aligned_slice.copy_from_slice(&[uniform_data]);
        vulkan.device.unmap_memory(uniform_buffer.memory);
        vulkan
            .device
            .bind_buffer_memory(uniform_buffer.buffer, uniform_buffer.memory, 0)
            .unwrap();

        uniform_buffer.size = buffer_create_info.size as u32;

        let uniform_descriptor = DescriptorInfo::new(
            vec![vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                p_immutable_samplers: ptr::null(),
            }],
            uniform_buffer,
            &vulkan,
        );

        let layout_create_info =
            vk::PipelineLayoutCreateInfo::builder().set_layouts(&uniform_descriptor.layouts);

        let pipeline_layout = vulkan
            .device
            .create_pipeline_layout(&layout_create_info, None)
            .unwrap();
            
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

        let pipeline = vulkan
            .device
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[graphic_pipeline_info.build()],
                None,
            )
            .expect("Unable to create graphics pipeline");

        //Destoy shader modules
        vulkan
            .device
            .destroy_shader_module(vertex_shader_module, None);
        vulkan
            .device
            .destroy_shader_module(fragment_shader_module, None);
        (
            pipeline[0],
            pipeline_layout,
            vertex_descriptor,
            uniform_descriptor,
        )
    }
}

pub fn create_uniform_data(swapchain: &Swapchain) -> UniformBufferObject {
    UniformBufferObject {
        model: Matrix4::<f32>::identity(),
        view: Matrix4::look_at(
            Point3::new(2.0, 2.0, 2.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        ),
        proj: cgmath::perspective(
            Deg(45.0),
            swapchain.extent.width as f32 / swapchain.extent.height as f32,
            0.1,
            10.0,
        ),
    }
}