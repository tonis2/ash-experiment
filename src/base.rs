use crate::modules;
use crate::utility;

use modules::surface::*;
use utility::helpers::*;

use ash::extensions::khr::{Surface, Swapchain};
use ash::{vk, Device, Entry, Instance};

use std::mem;

pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};

pub struct VulkanBase {
    pub entry: Entry,
    pub instance: Instance,
    pub device: Device,
    pub surface_loader: Surface,
    pub swapchain_loader: Swapchain,
    pub pdevice: vk::PhysicalDevice,
    pub device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub queue_family_index: u32,
    pub present_queue: vk::Queue,

    pub surface: vk::SurfaceKHR,
    pub surface_format: vk::SurfaceFormatKHR,
    pub surface_resolution: vk::Extent2D,

    pub swapchain: vk::SwapchainKHR,
    pub present_images: Vec<vk::Image>,
    pub present_image_views: Vec<vk::ImageView>,

    pub pool: vk::CommandPool,
    pub draw_command_buffer: vk::CommandBuffer,
    pub setup_command_buffer: vk::CommandBuffer,

    pub depth_image: vk::Image,
    pub depth_image_view: vk::ImageView,
    pub depth_image_memory: vk::DeviceMemory,

    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    pub current_frame: usize,
}

impl VulkanBase {
    pub fn new(window: &winit::window::Window) {
        unsafe {
            let (instance, surface, entry) = create_instance(&window);
            let surface_loader = Surface::new(&entry, &instance);
            let pdevices = instance
                .enumerate_physical_devices()
                .expect("Physical device error");
        }
    }
}
