mod pipeline;

use vulkan::{
    prelude::*, utilities::FPSLimiter, Context, Framebuffer, Queue, Swapchain, VkThread,
};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use pipeline::{Pipeline, Vertex};
use std::path::Path;
use std::sync::Arc;

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("test")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .expect("Failed to create window.");

    let vulkan = Arc::new(Context::new(&window, "load_model", true));
    let mut queue = Queue::new(vulkan.clone());

    let instance = VkThread::new(vulkan.clone());

    let swapchain = Swapchain::new(vulkan.clone());

    let mut pipeline = Pipeline::new(&swapchain, &instance);

    let (vertices, indices) = load_model(Path::new("assets/chalet.obj"));
    let index_buffer = instance.create_gpu_buffer(vk::BufferUsageFlags::INDEX_BUFFER, &indices);
    let vertex_buffer = instance.create_gpu_buffer(vk::BufferUsageFlags::VERTEX_BUFFER, &vertices);

    let command_buffers = instance.create_command_buffers(swapchain.image_views.len());

    let framebuffers: Vec<Framebuffer> = swapchain
        .image_views
        .iter()
        .map(|image| {
            Framebuffer::new(
                vk::FramebufferCreateInfo::builder()
                    .layers(1)
                    .render_pass(pipeline.renderpass)
                    .attachments(&[*image, pipeline.depth_image.view()])
                    .width(swapchain.width())
                    .height(swapchain.height())
                    .build(),
                vulkan.clone(),
            )
        })
        .collect();

    let mut tick_counter = FPSLimiter::new();

    let extent = vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: swapchain.extent,
    };

    //Let's prebuild command buffers in this example
    for (image_index, _image) in swapchain.image_views.iter().enumerate() {
        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: swapchain.extent.width as f32,
            height: swapchain.extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .framebuffer(framebuffers[image_index].buffer())
            .render_pass(pipeline.renderpass)
            .clear_values(&[
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
            ])
            .render_area(extent)
            .build();

        instance.build_command(
            command_buffers[image_index],
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
                    &[pipeline.pipeline_descriptor.set],
                    &[],
                );
                device.cmd_set_viewport(command_buffer, 0, &viewports);
                device.cmd_set_scissor(command_buffer, 0, &[extent]);
                device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer.buffer], &[0]);
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
    }

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
            pipeline.uniform_transform.model = cgmath::Matrix4::from_axis_angle(
                cgmath::Vector3::new(0.0, 0.0, 1.0),
                cgmath::Deg(90.0) * delta_time,
            ) * pipeline.uniform_transform.model;

            pipeline
                .uniform_buffer
                .upload_to_buffer(&[pipeline.uniform_transform], 0);

            let frame = queue.load_next_frame(&swapchain);

            if let Ok((image_index, _is_suboptimal)) = frame {
                queue.render_frame(
                    &swapchain,
                    command_buffers[image_index as usize],
                    image_index,
                );
            } else {
                println!("Failed to draw frame {:?}", frame.err());

                println!("Need to implement resize of swapchain here!")
            }
        }
        Event::LoopDestroyed => {}
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
