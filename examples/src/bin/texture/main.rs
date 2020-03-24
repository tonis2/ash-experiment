mod pipeline;
mod renderpass;

use vulkan::{
    prelude::*, utilities::FPSLimiter, Context, Framebuffer, Queue, Swapchain, VkInstance,
};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use pipeline::{Pipeline, Vertex};
use std::sync::Arc;

fn main() {
    let vertices = vec![
        Vertex {
            pos: [-0.75, -0.75],
            color: [1.0, 0.0, 0.0],
            tex_coord: [1.0, 0.0],
        },
        Vertex {
            pos: [0.75, -0.75],
            color: [0.0, 1.0, 0.0],
            tex_coord: [0.0, 0.0],
        },
        Vertex {
            pos: [0.75, 0.75],
            color: [0.0, 0.0, 1.0],
            tex_coord: [0.0, 1.0],
        },
        Vertex {
            pos: [-0.75, 0.75],
            color: [1.0, 1.0, 1.0],
            tex_coord: [1.0, 1.0],
        },
    ];

    let indices = vec![0, 1, 2, 2, 3, 0];

    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("test")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .expect("Failed to create window.");

    let vulkan = Arc::new(Context::new(&window, "triangle", true));

    let instance = VkInstance::new(vulkan.clone());

    let mut queue = Queue::new(vulkan.clone());

    let swapchain = Swapchain::new(vulkan.clone(), &window);
    let render_pass = renderpass::create_render_pass(&swapchain, &instance);

    let pipeline = Pipeline::create_pipeline(&swapchain, render_pass, &instance);

    let index_buffer = instance.create_gpu_buffer(vk::BufferUsageFlags::INDEX_BUFFER, &indices);
    let vertex_buffer = instance.create_gpu_buffer(vk::BufferUsageFlags::VERTEX_BUFFER, &vertices);

    let command_buffers = instance.create_command_buffers(swapchain.image_views.len());
    let framebuffers: Vec<Framebuffer> = swapchain
        .image_views
        .iter()
        .map(|image| swapchain.build_framebuffer(render_pass, vec![*image]))
        .collect();

    let mut tick_counter = FPSLimiter::new();

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
            print!("FPS: {}\r", tick_counter.fps());

            tick_counter.tick_frame();
        }
        Event::RedrawRequested(_window_id) => {
            let extent = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swapchain.extent,
            };

            let viewports = [vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: swapchain.extent.width as f32,
                height: swapchain.extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }];

            let next_frame = queue.next_frame(&swapchain);

            let render_pass_info = vk::RenderPassBeginInfo::builder()
                .framebuffer(framebuffers[next_frame.image_index].buffer())
                .render_pass(render_pass)
                .clear_values(&[vk::ClearValue {
                    // clear value for color buffer
                    color: vk::ClearColorValue {
                        float32: [0.0, 0.0, 0.0, 1.0],
                    },
                }])
                .render_area(extent)
                .build();

            instance.build_command(
                command_buffers[next_frame.image_index],
                |command_buffer, device| unsafe {
                    device.cmd_begin_render_pass(
                        command_buffer,
                        &render_pass_info,
                        vk::SubpassContents::INLINE,
                    );
                    device.cmd_bind_pipeline(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        pipeline.pipeline,
                    );
                    device.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        pipeline.layout,
                        0,
                        &[pipeline.descriptor_set],
                        &[],
                    );
                    device.cmd_set_viewport(command_buffer, 0, &viewports);
                    device.cmd_set_scissor(command_buffer, 0, &[extent]);
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
                    device.cmd_draw_indexed(command_buffer, indices.len() as u32, 1, 0, 0, 1);
                    device.cmd_end_render_pass(command_buffer);
                },
            );

            queue.render_frame(
                &next_frame,
                &swapchain,
                command_buffers[next_frame.image_index],
                vulkan.clone(),
            );
        }
        Event::LoopDestroyed => {}
        _ => {}
    });
}
