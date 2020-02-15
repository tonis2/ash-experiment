use crate::modules::{debug::create_debugger, device, surface};
use ash::{
    extensions::khr::Surface,
    version::{DeviceV1_0, InstanceV1_0},
    vk, Device, Entry, Instance,
};
use winit::window::Window;

#[allow(dead_code)]
pub struct VkInstance {
    entry: Entry,
    debugger: vk::DebugReportCallbackEXT,
    pub instance: Instance,
    pub surface: Surface,
    pub surface_khr: vk::SurfaceKHR,
    pub physical_device: vk::PhysicalDevice,
    pub device: Device,
    pub queue_family_index: u32,
    pub queue: vk::Queue,
}

impl VkInstance {
    pub fn new(window: &Window) -> Self {
        unsafe {
            let (entry, instance) = crate::utility::create_entry();
            let debugger = create_debugger(&entry, &instance);
            let surface = Surface::new(&entry, &instance);
            let surface_khr = surface::create_surface(&entry, &instance, &window).unwrap();

            let (pdevice, queue_family_index) =
                device::create_physical_device(&instance, &surface, surface_khr);
            let (device, queue) = device::create_device(queue_family_index, &instance, pdevice);

            Self {
                entry,
                debugger,
                instance,
                surface,
                surface_khr,
                physical_device: pdevice,
                device: device,
                queue_family_index,
                queue,
            }
        }
    }
}

impl VkInstance {
    pub fn get_physical_device_memory_properties(&self) -> vk::PhysicalDeviceMemoryProperties {
        unsafe {
            self.instance
                .get_physical_device_memory_properties(self.physical_device)
        }
    }

    pub fn wait_idle(&self) -> std::result::Result<(), vk::Result> {
        unsafe { self.device.device_wait_idle() }
    }
}

impl Drop for VkInstance {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
            self.device.destroy_device(None);
        }
    }
}
