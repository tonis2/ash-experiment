mod pipelines;
use ash::{version::DeviceV1_0, vk};
use vulkan::*;

use pipelines::Vertex;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

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
        .with_inner_size(winit::dpi::LogicalSize::new(1500.0, 800.0))
        .build(&event_loop)
        .expect("Failed to create window.");

    let mut vulkan_base = VkInstance::new(&window);
    let command_pool = vulkan_base.create_command_pool();
    let swapchain = Swapchain::new(&vulkan_base, 1500, 800);

    let descriptor_pool = vulkan_base.create_descriptor_pool(3);

    let render_pass = swapchain.create_render_pass(&vulkan_base.device);
    let frame_buffers = swapchain.create_frame_buffers(&render_pass, &vulkan_base);

    //Create pipeline
    let (pipeline, layout, vertex_descriptor, uniform_descriptor) =
        pipelines::create_pipeline(&swapchain, render_pass, &vulkan_base);

    let mut index_buffer = create_index_buffer(&indices, &vulkan_base);
    let mut vertex_buffer = create_vertex_buffer(&vertices, &vulkan_base, &vertex_descriptor);

    let uniform_descriptor_sets = uniform_descriptor.build(&vulkan_base, &descriptor_pool, 1);

    let command_buffers =
        vulkan_base.create_command_buffers(command_pool, swapchain.image_views.len());

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

            let frame = vulkan_base.build_frame(
                &command_buffers,
                &frame_buffers,
                &render_pass,
                extent[0],
                &swapchain,
                clear_values,
                |command_buffer, device| unsafe {
                    device.cmd_bind_pipeline(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        pipeline,
                    );
                    device.cmd_set_viewport(command_buffer, 0, &viewports);
                    device.cmd_set_scissor(command_buffer, 0, &extent);
                    device.cmd_bind_vertex_buffers(
                        command_buffer,
                        0,
                        &[vertex_buffer.buffer],
                        &[0],
                    );
                    device.cmd_bind_index_buffer(
                        command_buffer,
                        index_buffer.buffer,
                        0,
                        vk::IndexType::UINT32,
                    );
                    device.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        layout,
                        0,
                        &uniform_descriptor_sets,
                        &[],
                    );
                    device.cmd_draw_indexed(command_buffer, index_buffer.size, 1, 0, 0, 1);
                },
            );
            vulkan_base.render_frame(frame, &swapchain);
        }
        Event::LoopDestroyed => unsafe {
            vulkan_base.wait_idle().unwrap();
            for &framebuffer in frame_buffers.iter() {
                vulkan_base.device.destroy_framebuffer(framebuffer, None);
            }

            vulkan_base.device.destroy_command_pool(command_pool, None);
            vulkan_base.device.destroy_render_pass(render_pass, None);
            vulkan_base.device.destroy_pipeline(pipeline, None);
            vulkan_base.device.destroy_pipeline_layout(layout, None);
            vertex_buffer.destroy(&vulkan_base);
            index_buffer.destroy(&vulkan_base);
            swapchain.destroy(&vulkan_base);
        },
        _ => {}
    });
}
