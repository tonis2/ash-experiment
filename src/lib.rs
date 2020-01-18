mod modules;
mod definitions;

use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};


// Constants

pub trait Base: 'static + Sized {
    fn init() -> (Self, winit::window::WindowBuilder);
    fn update();
    fn render();
}

pub fn app<E: Base>() {
    let event_loop = EventLoop::new();
    let (app_ref, window_builder) = E::init();

    let window = window_builder
        .build(&event_loop)
        .expect("Failed to create window.");
        
    let app_info = definitions::VulkanInfo::default();    
    let entry = ash::Entry::new().unwrap();
    let instance = modules::instance::create_instance(&app_info, &entry);    

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
                        dbg!();
                        *control_flow = ControlFlow::Exit
                    }
                    _ => {}
                },
            },
            _ => {}
        },
        _ => (),
    })
}
