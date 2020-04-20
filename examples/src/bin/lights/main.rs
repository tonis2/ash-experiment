mod pipelines;

use vulkan::{
    prelude::*,
    utilities::{as_byte_slice, Batch, FPSLimiter, Mesh},
    Context, Framebuffer, Queue, Swapchain, VkThread,
};

use examples::utils::{events, Camera};
use pipelines::{mesh_pipeline, Light, PushConstantModel, Vertex};
use std::path::Path;
use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("test")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .expect("Failed to create window.");

    let mut camera = Camera::new(cgmath::Point3::new(0.0, 0.0, 0.0), 15.0, 1.3);

    let light = Light::new(
        cgmath::Vector4::new(6.0, 7.0, 5.0, 1.0),
        [0.8, 0.8, 0.8, 1.0],
        [0.5, 0.5, 0.5, 1.0],
    );

    let vulkan = Arc::new(Context::new(&window, "lights", true));
    let instance = VkThread::new(vulkan.clone());

    let mut queue = Queue::new(vulkan.clone());

    let mut swapchain = Swapchain::new(vulkan.clone());
    let command_buffers = instance.create_command_buffers(swapchain.image_views.len());

    let mut pipeline = mesh_pipeline::Pipeline::new(&swapchain, &instance, camera, light);

    let mut framebuffers: Vec<Framebuffer> = swapchain
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

    let mut shadow_framebuffer: Framebuffer = Framebuffer::new(
        vk::FramebufferCreateInfo::builder()
            .layers(1)
            .render_pass(pipeline.shadow_pipeline.renderpass)
            .attachments(&[pipeline.shadow_pipeline.image.view()])
            .width(swapchain.width())
            .height(swapchain.height())
            .build(),
        vulkan.clone(),
    );

    let scene_batch = load_model(Path::new("assets/lights.obj"));

    let scene_index_buffer =
        instance.create_gpu_buffer(vk::BufferUsageFlags::INDEX_BUFFER, &scene_batch.indices);
    let scene_vertex_buffer =
        instance.create_gpu_buffer(vk::BufferUsageFlags::VERTEX_BUFFER, &scene_batch.vertices);

    let ball_batch = load_model(Path::new("assets/ball.obj"));

    let ball_index_buffer =
        instance.create_gpu_buffer(vk::BufferUsageFlags::INDEX_BUFFER, &ball_batch.indices);

    let ball_vertex_buffer =
        instance.create_gpu_buffer(vk::BufferUsageFlags::VERTEX_BUFFER, &ball_batch.vertices);

    let mut tick_counter = FPSLimiter::new();
    let mut events = events::Event::new();

    let mut scene_data = PushConstantModel::new(
        cgmath::Decomposed {
            scale: 1.0,
            rot: cgmath::Rotation3::from_angle_x(cgmath::Deg(0.0)),
            disp: cgmath::Vector3::new(0.0, 0.0, 0.0),
        },
        [0.6, 0.5, 0.5],
    );

    let ball_data = PushConstantModel::new(
        cgmath::Decomposed {
            scale: 0.2,
            rot: cgmath::Rotation3::from_angle_x(cgmath::Deg(0.0)),
            disp: cgmath::Vector3::new(light.position.x, light.position.y, light.position.z),
        },
        [1.0, 1.0, 1.0],
    );

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(_) => {}
            _ => {
                events.handle_event(event);
                if events.event_happened {
                    //Camera updates
                    camera.handle_events(&events);
                    pipeline.uniform_buffer.upload_to_buffer(&[camera.raw()], 0);
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
            let delta_time = tick_counter.delta_time();
            // rotate scene

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

            use cgmath::Zero;
            let transform: cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Basis3<f32>> =
                cgmath::Decomposed {
                    scale: 1.0,
                    rot: cgmath::Rotation3::from_angle_y(cgmath::Deg(90.0) * delta_time),
                    disp: cgmath::Vector3::zero(),
                };

            scene_data.update_transform(transform);

            if let Ok((image_index, _s)) = queue.load_next_frame(&mut swapchain) {
                let scene_pass = vk::RenderPassBeginInfo::builder()
                    .framebuffer(framebuffers[image_index as usize].buffer())
                    .render_pass(pipeline.renderpass)
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

                let shadow_pass_info = vk::RenderPassBeginInfo::builder()
                    .framebuffer(shadow_framebuffer.buffer())
                    .render_pass(pipeline.shadow_pipeline.renderpass)
                    .render_area(extent)
                    .clear_values(&[vk::ClearValue {
                        // clear value for depth buffer
                        depth_stencil: vk::ClearDepthStencilValue {
                            depth: 1.0,
                            stencil: 0,
                        },
                    }])
                    .build();

                instance.build_command(
                    command_buffers[image_index as usize],
                    |command_buffer, device| unsafe {
                        device.cmd_set_viewport(command_buffer, 0, &viewports);
                        device.cmd_set_scissor(command_buffer, 0, &[extent]);
                        //Depth buffer
                        device.cmd_push_constants(
                            command_buffer,
                            pipeline.layout,
                            vk::ShaderStageFlags::VERTEX,
                            0,
                            as_byte_slice(&scene_data),
                        );

                        device.cmd_begin_render_pass(
                            command_buffer,
                            &shadow_pass_info,
                            vk::SubpassContents::INLINE,
                        );

                        device.cmd_bind_pipeline(
                            command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            pipeline.shadow_pipeline.pipeline,
                        );
                        device.cmd_bind_descriptor_sets(
                            command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            pipeline.shadow_pipeline.layout,
                            0,
                            &[pipeline.shadow_pipeline.descriptor.set],
                            &[],
                        );

                        device.cmd_bind_vertex_buffers(
                            command_buffer,
                            0,
                            &[scene_vertex_buffer.buffer],
                            &[0],
                        );
                        device.cmd_bind_index_buffer(
                            command_buffer,
                            scene_index_buffer.buffer,
                            0,
                            vk::IndexType::UINT32,
                        );
                        device.cmd_draw_indexed(
                            command_buffer,
                            scene_batch.indices.len() as u32,
                            1,
                            0,
                            0,
                            1,
                        );

                        device.cmd_end_render_pass(command_buffer);

                        //Scene
                        device.cmd_begin_render_pass(
                            command_buffer,
                            &scene_pass,
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

                        device.cmd_bind_vertex_buffers(
                            command_buffer,
                            0,
                            &[scene_vertex_buffer.buffer],
                            &[0],
                        );
                        device.cmd_bind_index_buffer(
                            command_buffer,
                            scene_index_buffer.buffer,
                            0,
                            vk::IndexType::UINT32,
                        );
                        device.cmd_draw_indexed(
                            command_buffer,
                            scene_batch.indices.len() as u32,
                            1,
                            0,
                            0,
                            1,
                        );

                        //Ball
                        device.cmd_push_constants(
                            command_buffer,
                            pipeline.layout,
                            vk::ShaderStageFlags::VERTEX,
                            0,
                            as_byte_slice(&ball_data),
                        );
                        device.cmd_bind_vertex_buffers(
                            command_buffer,
                            0,
                            &[ball_vertex_buffer.buffer],
                            &[0],
                        );
                        device.cmd_bind_index_buffer(
                            command_buffer,
                            ball_index_buffer.buffer,
                            0,
                            vk::IndexType::UINT32,
                        );
                        device.cmd_draw_indexed(
                            command_buffer,
                            ball_batch.indices.len() as u32,
                            1,
                            0,
                            0,
                            1,
                        );

                        device.cmd_end_render_pass(command_buffer);
                    },
                );

                queue.render_frame(
                    &mut swapchain,
                    command_buffers[image_index as usize],
                    image_index,
                );
            } else {
                println!("Failed to draw frame");
                //Resize
                vulkan.wait_idle();
                swapchain = Swapchain::new(vulkan.clone());

                pipeline = mesh_pipeline::Pipeline::new(&swapchain, &instance, camera, light);
                framebuffers = swapchain
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

                shadow_framebuffer = Framebuffer::new(
                    vk::FramebufferCreateInfo::builder()
                        .layers(1)
                        .render_pass(pipeline.shadow_pipeline.renderpass)
                        .attachments(&[pipeline.shadow_pipeline.image.view()])
                        .width(swapchain.width())
                        .height(swapchain.height())
                        .build(),
                    vulkan.clone(),
                );
            }
        }
        Event::LoopDestroyed => {}
        _ => {}
    });
}

fn load_model(model_path: &Path) -> Batch<Vertex> {
    let model_obj = tobj::load_obj(model_path).expect("Failed to load model object!");

    let (models, _) = model_obj;
    let mut batch = Batch::<Vertex>::new();

    for m in models.iter() {
        let mesh = &m.mesh;
        let mut mesh_data = Mesh::default();

        for i in 0..mesh.positions.len() / 3 {
            let vertex = Vertex {
                pos: [
                    mesh.positions[i * 3],
                    mesh.positions[i * 3 + 1],
                    mesh.positions[i * 3 + 2],
                ],
                normal: [
                    mesh.normals[i * 3],
                    mesh.normals[i * 3 + 1],
                    mesh.normals[i * 3 + 2],
                ],
            };
            mesh_data.vertices.push(vertex);
        }

        mesh_data.indices = mesh.indices.clone();
        batch.add(&mut mesh_data);
    }

    batch
}
