mod pipeline;
mod renderpass;

use vulkan::{
    prelude::*,
    utilities::{FPSLimiter},
    Context, Queue, Swapchain, VkInstance,
};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use pipeline::{Pipeline, Vertex};
use std::sync::Arc;

fn vertex(pos: [i8; 3], tc: [i8; 2]) -> Vertex {
    Vertex {
        pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32],
        tex_cord: [tc[0] as f32, tc[1] as f32],
    }
}
fn main() {
    //Cube data
    let vertices = vec![
        vertex([-1, -1, 1], [0, 0]),
        vertex([1, -1, 1], [1, 0]),
        vertex([1, 1, 1], [1, 1]),
        vertex([-1, 1, 1], [0, 1]),
        // bottom (0, 0, -1)
        vertex([-1, 1, -1], [1, 0]),
        vertex([1, 1, -1], [0, 0]),
        vertex([1, -1, -1], [0, 1]),
        vertex([-1, -1, -1], [1, 1]),
        // right (1, 0, 0)
        vertex([1, -1, -1], [0, 0]),
        vertex([1, 1, -1], [1, 0]),
        vertex([1, 1, 1], [1, 1]),
        vertex([1, -1, 1], [0, 1]),
        // left (-1, 0, 0)
        vertex([-1, -1, 1], [1, 0]),
        vertex([-1, 1, 1], [0, 0]),
        vertex([-1, 1, -1], [0, 1]),
        vertex([-1, -1, -1], [1, 1]),
        // front (0, 1, 0)
        vertex([1, 1, -1], [1, 0]),
        vertex([-1, 1, -1], [0, 0]),
        vertex([-1, 1, 1], [0, 1]),
        vertex([1, 1, 1], [1, 1]),
        // back (0, -1, 0)
        vertex([1, -1, 1], [0, 0]),
        vertex([-1, -1, 1], [1, 0]),
        vertex([-1, -1, -1], [1, 1]),
        vertex([1, -1, -1], [0, 1]),
    ];

    let indices = vec![
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("test")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .expect("Failed to create window.");

    let vulkan = Arc::new(Context::new(&window));
    let mut queue = Queue::new(vulkan.clone());

    let instance = VkInstance::new(vulkan.clone());

    let swapchain = Swapchain::new(vulkan.clone(), &window);
    let render_pass = renderpass::create_render_pass(&swapchain, &instance);

    let mut pipeline = Pipeline::create_pipeline(&swapchain, render_pass, &instance);

    let command_buffers = instance.create_command_buffers(swapchain.image_views.len());

    let mut tick_counter = FPSLimiter::new();

    let index_buffer = instance.create_gpu_buffer(vk::BufferUsageFlags::INDEX_BUFFER, &indices);
    let vertex_buffer = instance.create_gpu_buffer(vk::BufferUsageFlags::VERTEX_BUFFER, &vertices);

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
            let delta_time = tick_counter.delta_time();
            
            //Update uniform buffer
            pipeline.uniform_transform.model = cgmath::Matrix4::from_axis_angle(
                cgmath::Vector3::new(0.0, 0.0, 1.0),
                cgmath::Deg(90.0) * delta_time,
            ) * pipeline.uniform_transform.model;
    
            pipeline.uniform_buffer
                .upload_to_buffer(&[pipeline.uniform_transform.clone()], 0);

            let extent = [vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swapchain.extent,
            }];

            let clear_values = [
                vk::ClearValue {
                    // clear value for color buffer
                    color: vk::ClearColorValue {
                        float32: [0.0, 0.0, 0.0, 1.0],
                    },
                },
                vk::ClearValue {
                    // clear value for depth buffer
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                },
            ];

            let viewports = [vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: swapchain.extent.width as f32,
                height: swapchain.extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }];

            let next_frame = queue.next_frame(&swapchain);

            vulkan.build_command(
                command_buffers[next_frame.image_index],
                extent[0],
                &clear_values,
                vec![
                    swapchain.get_image(next_frame.image_index),
                    pipeline.depth_image.1,
                ],
                render_pass,
                &swapchain,
                |command_buffer, device| unsafe {
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
                        &pipeline.descriptors,
                        &[],
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
                    device.cmd_draw_indexed(command_buffer, indices.len() as u32, 1, 0, 0, 1);
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
