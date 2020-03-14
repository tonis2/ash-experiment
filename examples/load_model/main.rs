mod pipeline;
mod renderpass;

use std::path::Path;
use vulkan::{utilities::FPSLimiter, Context, Queue, Swapchain, VkInstance};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use ash::{version::DeviceV1_0, vk};
use pipeline::{Pipeline, Vertex};
use std::sync::Arc;

fn main() {
    let (vertices, indices) = load_model(Path::new("examples/assets/chalet.obj"));

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

    let frame_buffers =
        swapchain.create_frame_buffers(&render_pass, vec![pipeline.depth_image.1], vulkan.clone());
    let index_buffer =
        instance.create_device_local_buffer(vk::BufferUsageFlags::INDEX_BUFFER, &indices);
    let vertex_buffer =
        instance.create_device_local_buffer(vk::BufferUsageFlags::VERTEX_BUFFER, &vertices);

    let command_buffers = instance.create_command_buffers(swapchain.image_views.len());

    let mut tick_counter = FPSLimiter::new();

    let extent = vec![vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: swapchain.extent,
    }];

    let clear_values = vec![
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

    queue.build_frame(
        &command_buffers,
        &frame_buffers,
        &render_pass,
        extent[0],
        clear_values,
        vulkan.clone(),
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
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer.buffer], &[0]);
            device.cmd_bind_index_buffer(
                command_buffer,
                index_buffer.buffer,
                0,
                vk::IndexType::UINT32,
            );
            device.cmd_draw_indexed(command_buffer, indices.len() as u32, 1, 0, 0, 0);
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
            print!("FPS: {}\r", tick_counter.fps());
            tick_counter.tick_frame();
        }
        Event::RedrawRequested(_window_id) => {
            let delta_time = tick_counter.delta_time();

            pipeline.update_uniform_buffer(delta_time);
            let frame = queue.next_frame(&vulkan, &swapchain);
            queue.render_frame(&frame, &swapchain, &command_buffers, vulkan.clone());
        }
        Event::LoopDestroyed => unsafe {
            vulkan.wait_idle();

            for &framebuffer in frame_buffers.iter() {
                vulkan.device.destroy_framebuffer(framebuffer, None);
            }
        },
        _ => {}
    });
}

fn load_model(model_path: &Path) -> (Vec<Vertex>, Vec<u32>) {
    let model_obj = tobj::load_obj(model_path).expect("Failed to load model object!");

    let mut vertices = vec![];
    let mut indices = vec![];

    let (models, _) = model_obj;
    for m in models.iter() {
        let mesh = &m.mesh;

        if mesh.texcoords.len() == 0 {
            panic!("Missing texture coordinate for the model.")
        }

        let total_vertices_count = mesh.positions.len() / 3;
        for i in 0..total_vertices_count {
            let vertex = Vertex {
                pos: [
                    mesh.positions[i * 3],
                    mesh.positions[i * 3 + 1],
                    mesh.positions[i * 3 + 2],
                    1.0,
                ],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coord: [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]],
            };
            vertices.push(vertex);
        }

        indices = mesh.indices.clone();
    }

    (vertices, indices)
}
