mod definitions;
mod modules;

use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;

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

    let (debug_utils_loader, debug_merssager) =
        modules::debug::setup_debug_utils(app_info.validation_info.is_enable, &entry, &instance);

    let surface_stuff = modules::surface::create_surface(&entry, &instance, &window, 800, 600);

    let physical_device = modules::device::pick_physical_device(&instance, &surface_stuff);
    let (device, family_indices) = modules::device::create_logical_device(
        &instance,
        physical_device,
        &app_info,
        &surface_stuff,
    );

    let graphics_queue =
        unsafe { device.get_device_queue(family_indices.graphics_family.unwrap(), 0) };
    let present_queue =
        unsafe { device.get_device_queue(family_indices.present_family.unwrap(), 0) };

    let swapchain_stuff = crate::modules::swapchain::create_swapchain(
        &instance,
        &device,
        physical_device,
        &surface_stuff,
        &family_indices,
    );

    let swapchain_imageviews = crate::modules::swapchain::create_image_views(
        &device,
        swapchain_stuff.swapchain_format,
        &swapchain_stuff.swapchain_images,
    );

    let render_pass =
        crate::modules::pipeline::create_render_pass(&device, swapchain_stuff.swapchain_format);

    let _pipeline =
        crate::modules::pipeline::create_graphics_pipeline(&device, render_pass, &swapchain_stuff);

    let _pipeline =
        crate::modules::pipeline::create_graphics_pipeline(&device, render_pass, &swapchain_stuff);

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
