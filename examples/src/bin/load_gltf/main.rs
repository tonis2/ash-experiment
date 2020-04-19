mod pipelines;
use vulkan::{
    prelude::*, utilities::as_byte_slice, utilities::FPSLimiter, Context, Framebuffer, Queue,
    Swapchain, VkThread,
};

use examples::utils::{gltf_importer, events};
use pipelines::{definitions::PushTransform, mesh_pipeline};
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

    let mut scene =
        gltf_importer::Importer::load(Path::new("assets/gltf_texture.gltf")).build(&instance);

    let mut mesh_pipeline = mesh_pipeline::Pipeline::build_for(&scene, &swapchain, &instance);

    let command_buffers = instance.create_command_buffers(swapchain.image_views.len());
    let mut framebuffers: Vec<Framebuffer> = swapchain
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

    let mut tick_counter = FPSLimiter::new();
    let mut events = events::Event::new();
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::DroppedFile(path) => {
                //Drop GLTF file on running window to load new file
                println!("Loading model at {:?}", path);
                scene = gltf_importer::Importer::load(&path).build(&instance);
                mesh_pipeline = mesh_pipeline::Pipeline::build_for(&scene, &swapchain, &instance);
                framebuffers = swapchain
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
            }
            _ => {
                events.handle_event(event);
                if events.event_happened {
                    //Camera updates
                    mesh_pipeline.camera.handle_events(&events);
                    mesh_pipeline
                        .uniform_buffer
                        .upload_to_buffer(&[mesh_pipeline.camera.raw()], 0);
                    events.clear();
                }
            }
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

                        for node in &scene.nodes {
                            if let Some(mesh_index) = node.mesh_index {
                                let mesh = scene.get_mesh(mesh_index);

                                device.cmd_push_constants(
                                    command_buffer,
                                    mesh_pipeline.layout,
                                    vk::ShaderStageFlags::VERTEX,
                                    0,
                                    as_byte_slice(&PushTransform {
                                        transform: node.transform_matrix,
                                    }),
                                );

                                device.cmd_bind_descriptor_sets(
                                    command_buffer,
                                    vk::PipelineBindPoint::GRAPHICS,
                                    mesh_pipeline.layout,
                                    0,
                                    &[mesh_pipeline.pipeline_descriptor.set],
                                    &[],
                                );

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
                                    device.cmd_draw_indexed(
                                        command_buffer,
                                        scene.indices_len,
                                        1,
                                        0,
                                        0,
                                        1,
                                    );
                                });
                            }
                        }

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
                swapchain = Swapchain::new(vulkan.clone());
                mesh_pipeline = mesh_pipeline::Pipeline::build_for(&scene, &swapchain, &instance);
                framebuffers = swapchain
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
            }
        }
        Event::LoopDestroyed => {}
        _ => {}
    });
}
