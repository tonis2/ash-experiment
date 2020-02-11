use crate::modules::{debug::create_debugger, surface::create_surface, swapchain::*};
use ash::{
    extensions::khr::{Surface, Swapchain},
    version::{DeviceV1_0, InstanceV1_0},
    vk, Device, Entry, Instance,
};
use winit::window::Window;

const POSSIBLE_SAMPLE_COUNTS: [u32; 7] = [1, 2, 4, 8, 16, 32, 64];

pub struct VkInstance {
    entry: Entry,
    pub instance: Instance,
    pub surface: Surface,
    pub surface_khr: vk::SurfaceKHR,
    pub physical_device: vk::PhysicalDevice,
    pub device: Device,
    pub queue_family_index: u32,
    pub present_queue: vk::Queue,
}

impl VkInstance {
    pub fn new(window: &Window) -> Self {
        unsafe {
            let (entry, instance) = crate::utility::create_entry(&window);
            let debugger = create_debugger(&entry, &instance);
            let surface = Surface::new(&entry, &instance);
            let surface_khr = create_surface(&entry, &instance, &window).unwrap();
            let pdevices = instance
                .enumerate_physical_devices()
                .expect("Physical device error");
            let (pdevice, queue_family_index) = pdevices
                .iter()
                .map(|pdevice| {
                    instance
                        .get_physical_device_queue_family_properties(*pdevice)
                        .iter()
                        .enumerate()
                        .filter_map(|(index, ref info)| {
                            let supports_graphic_and_surface =
                                info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                    && surface
                                        .get_physical_device_surface_support(
                                            *pdevice,
                                            index as u32,
                                            surface_khr,
                                        )
                                        .unwrap();
                            if supports_graphic_and_surface {
                                Some((*pdevice, index))
                            } else {
                                None
                            }
                        })
                        .nth(0)
                })
                .filter_map(|v| v)
                .nth(0)
                .expect("Couldn't find suitable device.");
            let queue_family_index = queue_family_index as u32;

            let device_extension_names_raw = [Swapchain::name().as_ptr()];
            let features = vk::PhysicalDeviceFeatures {
                shader_clip_distance: 1,
                ..Default::default()
            };
            let priorities = [1.0];

            let queue_info = [vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priorities)
                .build()];

            let device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(&queue_info)
                .enabled_extension_names(&device_extension_names_raw)
                .enabled_features(&features);

            let device: Device = instance
                .create_device(pdevice, &device_create_info, None)
                .unwrap();

            let present_queue = device.get_device_queue(queue_family_index as u32, 0);

            Self {
                entry,
                instance,
                surface,
                surface_khr,
                physical_device: pdevice,
                device: device,
                queue_family_index,
                present_queue,
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
}

impl Drop for VkInstance {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}
