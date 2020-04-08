pub mod gltf_importer;
mod pipelines;
use vulkan::{
    prelude::*, utilities::FPSLimiter, Context, Framebuffer, Queue, Swapchain, VkInstance,
};

use std::{path::Path, sync::Arc};

use pipelines::{mesh_pipeline, Camera};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("test")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .expect("Failed to create window.");

    let vulkan = Arc::new(Context::new(&window, "gltf", true));
    let instance = VkInstance::new(vulkan.clone());
    let mut swapchain = Swapchain::new(vulkan.clone());
    let mut queue = Queue::new(vulkan.clone());

    let model = gltf_importer::Importer::load(Path::new("assets/gltf_test.gltf")).build(&instance);

    let camera = Camera::new(800.0 / 600.0);
    let mesh_pipeline = mesh_pipeline::Pipeline::new(&swapchain, camera, vulkan.clone());

    let command_buffers = instance.create_command_buffers(swapchain.image_views.len());
    let framebuffers: Vec<Framebuffer> = swapchain
        .image_views
        .iter()
        .map(|image| {
            Framebuffer::new(
                vk::FramebufferCreateInfo::builder()
                    .layers(1)
                    .render_pass(mesh_pipeline.renderpass)
                    .attachments(&[*image, mesh_pipeline.depth_image.view()])
                    .width(swapchain.width())
                    .height(swapchain.height())
                    .build(),
                vulkan.clone(),
            )
        })
        .collect();

    // let vertex_buffer =
    //     instance.create_gpu_buffer(vk::BufferUsageFlags::VERTEX_BUFFER, &model.vertices);
    // let index_buffer =
    //     instance.create_gpu_buffer(vk::BufferUsageFlags::INDEX_BUFFER, &model.indices);

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

            if let Ok((image_index, _s)) = queue.load_next_frame(&mut swapchain) {
                let scene_pass = vk::RenderPassBeginInfo::builder()
                    .framebuffer(framebuffers[image_index as usize].buffer())
                    .render_pass(mesh_pipeline.renderpass)
                    .render_area(extent)
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
                    .build();

                instance.build_command(
                    command_buffers[image_index as usize],
                    |command_buffer, device| unsafe {
                        device.cmd_set_viewport(command_buffer, 0, &viewports);
                        device.cmd_set_scissor(command_buffer, 0, &[extent]);
                        device.cmd_begin_render_pass(
                            command_buffer,
                            &scene_pass,
                            vk::SubpassContents::INLINE,
                        );
                        device.cmd_bind_pipeline(
                            command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            mesh_pipeline.pipeline,
                        );
                        device.cmd_bind_descriptor_sets(
                            command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            mesh_pipeline.layout,
                            0,
                            &[mesh_pipeline.pipeline_descriptor.set],
                            &[],
                        );

                        // device.cmd_bind_vertex_buffers(
                        //     command_buffer,
                        //     0,
                        //     &[vertex_buffer.buffer],
                        //     &[0],
                        // );
                        // device.cmd_bind_index_buffer(
                        //     command_buffer,
                        //     index_buffer.buffer,
                        //     0,
                        //     vk::IndexType::UINT32,
                        // );
                        // device.cmd_draw_indexed(
                        //     command_buffer,
                        //     model.indices.len() as u32,
                        //     1,
                        //     0,
                        //     0,
                        //     1,
                        // );

                        device.cmd_end_render_pass(command_buffer);
                    },
                );

                queue.render_frame(
                    &mut swapchain,
                    command_buffers[image_index as usize],
                    image_index,
                );
            }
        }
        Event::LoopDestroyed => {}
        _ => {}
    });
}
