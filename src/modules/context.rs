use ash::{
    extensions::khr::Surface,
    version::{DeviceV1_0, EntryV1_0, InstanceV1_0},
    vk, Device, Entry, Instance,
};

use super::{
    debug::{Debugger, ValidationInfo},
    device,
    queue::QueueFamilyIndices,
};
use crate::constants::*;
use super::platform::{create_surface, extension_names};

use std::ffi::CString;
use winit::window::Window;

pub struct Context {
    _entry: Entry,
    _debugger: Option<Debugger>,
    pub instance: Instance,
    pub surface: vk::SurfaceKHR,
    pub surface_loader: Surface,
    pub physical_device: vk::PhysicalDevice,
    pub device: Device,
    pub queue_family: QueueFamilyIndices,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub memory: vk_mem::Allocator,
    pub image_count: u32,
}

impl Context {
    pub fn new(window: &Window, app_name: &str, validation_enabled: bool) -> Self {
        let (entry, instance) = create_entry(app_name);

        let surface_loader = Surface::new(&entry, &instance);
        let surface =
            unsafe { create_surface(&entry, &instance, window).expect("Failed to create surface") };

        let physical_device =
            device::pick_physical_device(&instance, &surface_loader, surface, &DEVICE_EXTENSIONS);

        let validation: ValidationInfo = ValidationInfo {
            is_enable: validation_enabled,
            required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
        };

        let (device, queue) = device::create_logical_device(
            &instance,
            physical_device,
            &validation,
            &DEVICE_EXTENSIONS,
            &surface_loader,
            &surface,
        );

        let memory_info = vk_mem::AllocatorCreateInfo {
            physical_device: physical_device,
            device: device.clone(),
            instance: instance.clone(),
            ..Default::default()
        };

        let mut debugger: Option<Debugger> = None;
        if validation_enabled == true {
            debugger = Some(Debugger::new(&entry, &instance));
        }

        let capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .expect("Failed to query for surface capabilities.")
        };

        let image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count > 0 {
            image_count.min(capabilities.max_image_count)
        } else {
            image_count
        };

        unsafe {
            Context {
                _entry: entry,
                _debugger: debugger,
                instance,
                surface,
                surface_loader,
                physical_device,
                graphics_queue: device.get_device_queue(queue.graphics_family.unwrap(), 0),
                present_queue: device.get_device_queue(queue.present_family.unwrap(), 0),
                queue_family: queue,
                device,
                memory: vk_mem::Allocator::new(&memory_info).unwrap(),
                image_count,
            }
        }
    }

    pub fn find_depth_format(
        &self,
        candidate_formats: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> vk::Format {
        for &format in candidate_formats.iter() {
            let format_properties = unsafe {
                self.instance
                    .get_physical_device_format_properties(self.physical_device, format)
            };
            if tiling == vk::ImageTiling::LINEAR
                && format_properties.linear_tiling_features.contains(features)
            {
                return format.clone();
            } else if tiling == vk::ImageTiling::OPTIMAL
                && format_properties.optimal_tiling_features.contains(features)
            {
                return format.clone();
            }
        }

        panic!("Failed to find supported format!")
    }

    pub fn wait_idle(&self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("failed to wait device idle")
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            self.wait_idle();
            self.surface_loader.destroy_surface(self.surface, None);

            if self._debugger.is_some() {
                let debugger = self._debugger.as_ref().unwrap();
                debugger
                    .report_loader
                    .destroy_debug_report_callback(debugger.reporter, None);
            }

            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

//Create vulkan entry
pub fn create_entry(app_name: &str) -> (Entry, Instance) {
    let entry = Entry::new().unwrap();
    let app_name = CString::new(app_name).unwrap();

    let layer_names = [CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()];
    let layers_names_raw: Vec<*const i8> = layer_names
        .iter()
        .map(|raw_name| raw_name.as_ptr())
        .collect();

    let extension_names_raw = extension_names();

    let appinfo = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(0)
        .engine_name(&app_name)
        .engine_version(0)
        .api_version(vk::make_version(1, 0, 0));

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
