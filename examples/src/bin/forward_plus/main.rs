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
        .with_inner_size(winit::dpi::LogicalSize::new(900.0, 600.0))
        .build(&event_loop)
        .expect("Failed to create window.");

    let vulkan = Arc::new(Context::new(&window, "gltf", true));
    let instance = VkThread::new(vulkan.clone());
    let mut swapchain = Swapchain::new(vulkan.clone());
    let mut queue = Queue::new(vulkan.clone());

    //../../GLTF_tests/multi_texture.gltf
    let mut scene =
        gltf_importer::Importer::load(Path::new("assets/multi_texture.gltf")).build(&instance);

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
            } else {
                //Resize window
                vulkan.wait_idle();
            }
        }
        Event::LoopDestroyed => {}
        _ => {}
    });
}
