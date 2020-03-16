mod pipeline;
mod renderpass;

use vulkan::{
    prelude::*,
    utilities::{FPSLimiter, GLTFModel},
    Context, Queue, Swapchain, VkInstance,
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

    let vulkan = Arc::new(Context::new(&window));
    let mut queue = Queue::new(vulkan.clone());

    let instance = VkInstance::new(vulkan.clone());

    let swapchain = Swapchain::new(vulkan.clone(), &window);
    let render_pass = renderpass::create_render_pass(&swapchain, &instance);

    let mut pipeline = Pipeline::create_pipeline(&swapchain, render_pass, &instance);

    let model = GLTFModel::create_from(Path::new("examples/assets/model.gltf"));

    println!("{:?}", model.meshes);

    let command_buffers = instance.create_command_buffers(swapchain.image_views.len());

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
        Event::RedrawRequested(_window_id) => {}
        Event::LoopDestroyed => {}
        _ => {}
    });
}
