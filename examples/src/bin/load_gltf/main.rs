mod pipelines;

use vulkan::{
    prelude::*,
    utilities::{as_byte_slice, Batch, FPSLimiter, Mesh},
    Context, Framebuffer, Queue, Swapchain, VkInstance,
};

use std::path::Path;

use pipelines::{mesh_pipeline, Camera, Light, PushConstantModel, Vertex};
use std::sync::Arc;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("test")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .expect("Failed to create window.");

    let camera = Camera::new(800.0 / 600.0, cgmath::Point3::new(0.0, 8.0, 20.0));

    let light = Light::new(
        cgmath::Vector4::new(6.0, 7.0, 5.0, 1.0),
        [0.8, 0.8, 0.8, 1.0],
        [0.5, 0.5, 0.5, 1.0],
    );
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
