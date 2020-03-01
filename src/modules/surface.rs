use crate::utilities::platform::create_surface;
use ash::{
    extensions::khr::Surface,
    version::{EntryV1_0, InstanceV1_0},
    vk,
};
use winit::window::Window;

pub struct VkSurface {
    pub surface_loader: ash::extensions::khr::Surface,
    pub surface: vk::SurfaceKHR,
    pub width: u32,
    pub height: u32,
}

impl VkSurface {
    pub fn new<I: InstanceV1_0, E: EntryV1_0>(window: &Window, instance: &I, entry: &E) -> Self {
        let surface_loader = Surface::new(entry, instance);
        let window_size = window.inner_size();
        unsafe {
            let surface =
                create_surface(entry, instance, window).expect("Failed to create surface");
            Self {
                surface,
                surface_loader,
                width: window_size.width,
                height: window_size.height,
            }
        }
    }
}
