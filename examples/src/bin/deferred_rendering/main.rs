mod pipelines;
use vulkan::{
    prelude::*, utilities::as_byte_slice, utilities::FPSLimiter, Context, Queue, Swapchain,
    VkThread,
};

use examples::utils::{events, gltf_importer};

use pipelines::PushTransform;
use std::{path::Path, sync::Arc};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("test")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .expect("Failed to create window.");

    let vulkan = Arc::new(Context::new(&window, "gltf", true));
    let instance = VkThread::new(vulkan.clone());
    let mut swapchain = Swapchain::new(vulkan.clone());
    let mut queue = Queue::new(vulkan.clone());

    //../../GLTF_tests/multi_texture.gltf
    let mut scene = gltf_importer::Importer::load(Path::new("../../GLTF_tests/multi_texture.gltf"))
        .build(&instance);

    let g_buffer = pipelines::Gbuffer::build(&scene, &swapchain, &instance);
    let deferred_pipe =
        pipelines::Deferred::build(&g_buffer.get_buffer_images(), &swapchain, &instance);

    let command_buffers = instance.create_command_buffers(swapchain.image_views.len());
    let mut tick_counter = FPSLimiter::new();
    let mut events = events::Event::new();

    //Event loop
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::DroppedFile(path) => {
                //Drop GLTF file on running window to load new file
                println!("Loading model at {:?}", path);
                scene = gltf_importer::Importer::load(&path).build(&instance);
            }
            _ => {
                events.handle_event(event);
                if events.event_happened {
                    //Camera updates
                    events.clear();
                }
            }
        },
        Event::MainEventsCleared => {
            window.request_redraw();

            // print!("FPS: {}\r", tick_counter.fps());
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
                let g_pass = vk::RenderPassBeginInfo::builder()
                    .framebuffer(g_buffer.framebuffers[image_index as usize].buffer())
                    .render_pass(g_buffer.renderpass.pass())
                    .render_area(extent)
                    .clear_values(&[
                        vk::ClearValue {
                            color: vk::ClearColorValue {
                                float32: [0.0, 0.0, 0.0, 0.0],
                            },
                        },
                        vk::ClearValue {
                            color: vk::ClearColorValue {
                                float32: [0.0, 0.0, 0.0, 0.0],
                            },
                        },
                        vk::ClearValue {
                            color: vk::ClearColorValue {
                                float32: [0.0, 0.0, 0.0, 0.0],
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

                let deferred_pass = vk::RenderPassBeginInfo::builder()
                    .framebuffer(deferred_pipe.framebuffers[image_index as usize].buffer())
                    .render_pass(deferred_pipe.renderpass.pass())
                    .render_area(extent)
                    .clear_values(&[vk::ClearValue {
                        color: vk::ClearColorValue {
                            float32: [0.0, 0.0, 0.0, 0.0],
                        },
                    }])
                    .build();

                instance.build_command(
                    command_buffers[image_index as usize],
                    |command_buffer, device| unsafe {
                        device.cmd_set_viewport(command_buffer, 0, &viewports);

                        device.cmd_set_scissor(command_buffer, 0, &[extent]);

                        //Build gbuffer data
                        device.cmd_begin_render_pass(
                            command_buffer,
                            &g_pass,
                            vk::SubpassContents::INLINE,
                        );

                        device.cmd_bind_pipeline(
                            command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            g_buffer.pipeline.default(),
                        );

                        device.cmd_bind_descriptor_sets(
                            command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            g_buffer.pipeline.layout(),
                            0,
                            &[g_buffer.pipeline_descriptor.set],
                            &[],
                        );

                        for node in &scene.nodes {
                            if let Some(mesh_index) = node.mesh_index {
                                let mesh = scene.get_mesh(mesh_index);

                                mesh.primitives.iter().for_each(|primitive| {
                                    device.cmd_bind_vertex_buffers(
                                        command_buffer,
                                        0,
                                        &[scene.vertices.clone().buffer],
                                        &[primitive.vertex_offset as u64],
                                    );
                                    device.cmd_bind_index_buffer(
                                        command_buffer,
                                        scene.indices.clone().buffer,
                                        primitive.indice_offset as u64,
                                        vk::IndexType::UINT32,
                                    );

                                    device.cmd_push_constants(
                                        command_buffer,
                                        g_buffer.pipeline.layout(),
                                        vk::ShaderStageFlags::VERTEX,
                                        0,
                                        as_byte_slice(&PushTransform {
                                            transform: node.transform_matrix,
                                        }),
                                    );
                                    device.cmd_draw_indexed(
                                        command_buffer,
                                        primitive.indices_len as u32,
                                        1,
                                        0,
                                        0,
                                        0,
                                    );
                                });
                            }
                        }

                        device.cmd_end_render_pass(command_buffer);

                        //Draw quad as final render
                        device.cmd_begin_render_pass(
                            command_buffer,
                            &deferred_pass,
                            vk::SubpassContents::INLINE,
                        );

                        device.cmd_bind_pipeline(
                            command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            deferred_pipe.pipeline.default(),
                        );

                        device.cmd_bind_descriptor_sets(
                            command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            deferred_pipe.pipeline.layout(),
                            0,
                            &[deferred_pipe.pipeline_descriptor.set],
                            &[],
                        );
                        device.cmd_bind_vertex_buffers(
                            command_buffer,
                            0,
                            &[deferred_pipe.quad_vertex.buffer],
                            &[0],
                        );
                        device.cmd_bind_index_buffer(
                            command_buffer,
                            deferred_pipe.quad_index.buffer,
                            0,
                            vk::IndexType::UINT32,
                        );

                        device.cmd_draw_indexed(command_buffer, 6 as u32, 1, 0, 0, 0);
                        device.cmd_end_render_pass(command_buffer);
                    },
                );

                queue.render_frame(
                    &mut swapchain,
                    command_buffers[image_index as usize],
                    image_index,
                );
            } else {
                //Resize window
                vulkan.wait_idle();
            }
        }
        Event::LoopDestroyed => {}
        _ => {}
    });
}
