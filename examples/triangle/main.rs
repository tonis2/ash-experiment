mod pipeline;
mod renderpass;

use vulkan::{Swapchain, VkInstance};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use ash::{version::DeviceV1_0, vk};
use pipeline::Vertex;

fn main() {
    let vertices = vec![
        Vertex {
            pos: [-1.0, 1.0],
            color: [0.0, 1.0, 0.0, 1.0],
        },
        Vertex {
            pos: [1.0, 1.0],
            color: [0.0, 0.0, 1.0, 1.0],
        },
        Vertex {
            pos: [0.0, -1.0],
            color: [1.0, 0.0, 0.0, 1.0],
        },
    ];

    let indices = vec![0, 1, 2];

    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("test")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .expect("Failed to create window.");

    let mut vulkan = VkInstance::new(&window);

    let swapchain = Swapchain::new(&vulkan, &window);
    let render_pass = renderpass::create_render_pass(&swapchain, &vulkan.device);
    let frame_buffers = swapchain.create_frame_buffers(&render_pass, vec![], &vulkan);

    let (pipeline, layout) = pipeline::create_pipeline(&swapchain, render_pass, &vulkan);

    let index_buffer = pipeline::create_index_buffer(&indices, &vulkan);
    let vertex_buffer = pipeline::create_vertex_buffer(&vertices, &vulkan);

    let command_buffers = vulkan.create_command_buffers(swapchain.image_views.len());

    let extent = vec![vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: swapchain.extent,
    }];

    let clear_values = vec![vk::ClearValue {
        color: vk::ClearColorValue {
            float32: [0.0, 0.0, 0.0, 1.0],
        },
    }];

    let viewports = [vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: swapchain.extent.width as f32,
        height: swapchain.extent.height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    }];

    vulkan.build_frame(
        &command_buffers,
        &frame_buffers,
        &render_pass,
        extent[0],
        clear_values,
        |command_buffer, device| unsafe {
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);
            device.cmd_set_viewport(command_buffer, 0, &viewports);
            device.cmd_set_scissor(command_buffer, 0, &extent);
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer.buffer], &[0]);
            device.cmd_bind_index_buffer(
                command_buffer,
                index_buffer.buffer,
                0,
                vk::IndexType::UINT32,
            );
            device.cmd_draw_indexed(command_buffer, indices.len() as u32, 1, 0, 0, 1);
        },
    );

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => match input {
                KeyboardInput {
                    virtual_keycode,
                    state,
                    ..
                } => match (virtual_keycode, state) {
                    (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                        *control_flow = ControlFlow::Exit
                    }
                    _ => {}
                },
            },
            _ => {}
        },
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        Event::RedrawRequested(_window_id) => {
            let frame = vulkan.queue.next_frame(&vulkan, &swapchain);

            vulkan.render_frame(&frame, &swapchain, &command_buffers);
        }
        Event::LoopDestroyed => unsafe {
            vulkan.wait_idle();

            for &framebuffer in frame_buffers.iter() {
                vulkan.device.destroy_framebuffer(framebuffer, None);
            }

            swapchain.destroy(&vulkan);
            vulkan.device.destroy_render_pass(render_pass, None);
            vulkan.device.destroy_pipeline(pipeline, None);
            vulkan.device.destroy_pipeline_layout(layout, None);
            index_buffer.destroy();
            vertex_buffer.destroy();
        },
        _ => {}
    });
}
