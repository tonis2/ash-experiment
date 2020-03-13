use ash::{
    version::{DeviceV1_0, EntryV1_0, InstanceV1_0},
    vk, Device, Entry, Instance,
};

use super::{debug::Debugger, device, queue::QueueFamilyIndices, surface::VkSurface};

use crate::constants::*;
use crate::utilities::platform::extension_names;

use std::ffi::CString;
use winit::window::Window;

pub struct Context {
    _entry: Entry,
    _debugger: Debugger,
    pub instance: Instance,
    pub surface: VkSurface,
    pub physical_device: vk::PhysicalDevice,
    pub device: Device,
    pub queue_family: QueueFamilyIndices,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub memory: vk_mem::Allocator,
}

impl Context {
    pub fn new(window: &Window) -> Self {
        let (entry, instance) = create_entry();
        let debugger = Debugger::new(&entry, &instance);
        let surface = VkSurface::new(&window, &instance, &entry);
        let physical_device = device::pick_physical_device(&instance, &surface, &DEVICE_EXTENSIONS);
        let (device, queue) = device::create_logical_device(
            &instance,
            physical_device,
            &VALIDATION,
            &DEVICE_EXTENSIONS,
            &surface,
        );

        let memory_info = vk_mem::AllocatorCreateInfo {
            physical_device: physical_device,
            device: device.clone(),
            instance: instance.clone(),
            ..Default::default()
        };

       unsafe {
        Context {
            _entry: entry,
            _debugger: debugger,
            instance,
            surface,
            physical_device,
            graphics_queue: device.get_device_queue(queue.graphics_family.unwrap(), 0),
            present_queue: device.get_device_queue(queue.present_family.unwrap(), 0),
            queue_family: queue,
            device,
            memory: vk_mem::Allocator::new(&memory_info).unwrap(),
        }
       }
    }

    pub fn get_physical_device_memory_properties(&self) -> vk::PhysicalDeviceMemoryProperties {
        unsafe {
            self.instance
                .get_physical_device_memory_properties(self.physical_device)
        }
    }

    pub fn wait_idle(&self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("failed to wait device idle")
        }
    }

    pub fn destroy(&self) {
        unsafe {
            self.wait_idle();
            self.surface
                .surface_loader
                .destroy_surface(self.surface.surface, None);

            // self.queue.destroy(&self.device);
            self.device.destroy_device(None);
            // self._debugger.destroy();
            // self.instance.destroy_instance(None);
        }
    }

    // pub fn next_frame(&mut self) {
    //     self.queue.current_frame = (self.queue.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    // }
}

//Create vulkan entry
pub fn create_entry() -> (Entry, Instance) {
    let entry = Entry::new().unwrap();
    let app_name = CString::new(APP_NAME).unwrap();

    let layer_names = [CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()];
    let layers_names_raw: Vec<*const i8> = layer_names
        .iter()
        .map(|raw_name| raw_name.as_ptr())
        .collect();

    let extension_names_raw = extension_names();

    let appinfo = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(API_VERSION)
        .engine_name(&app_name)
        .engine_version(ENGINE_VERSION)
        .api_version(APPLICATION_VERSION);

    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&appinfo)
        .enabled_layer_names(&layers_names_raw)
        .enabled_extension_names(&extension_names_raw);
    unsafe {
        let instance: Instance = entry
            .create_instance(&create_info, None)
            .expect("Instance creation error");

        (entry, instance)
    }
}
