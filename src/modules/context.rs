use ash::{
    version::{DeviceV1_0, EntryV1_0, InstanceV1_0},
    vk, Device, Entry, Instance,
};

use super::{
    debug::{Debugger, ValidationInfo},
    device,
    queue::QueueFamilyIndices,
    surface::VkSurface,
};
use crate::constants::*;
use crate::utilities::platform::extension_names;
use std::ptr;

use std::ffi::CString;
use winit::window::Window;

pub struct Context {
    _entry: Entry,
    _debugger: Option<Debugger>,
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
    pub fn new(window: &Window, app_name: &str, validation_enabled: bool) -> Self {
        let (entry, instance) = create_entry(app_name);

        let surface = VkSurface::new(&window, &instance, &entry);
        let physical_device = device::pick_physical_device(&instance, &surface, &DEVICE_EXTENSIONS);
        let validation: ValidationInfo = ValidationInfo {
            is_enable: validation_enabled,
            required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
        };

        let (device, queue) = device::create_logical_device(
            &instance,
            physical_device,
            &validation,
            &DEVICE_EXTENSIONS,
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

    pub fn wait_idle(&self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("failed to wait device idle")
        }
    }

    pub fn create_descriptor(
        &self,
        images_count: u32,
        bindings: Vec<vk::DescriptorSetLayoutBinding>,
        descriptor_write_sets: &mut Vec<vk::WriteDescriptorSet>,
    ) -> (
        vk::DescriptorSetLayout,
        vk::DescriptorSet,
        vk::DescriptorPool,
    ) {
        let pool_sizes: Vec<vk::DescriptorPoolSize> = bindings
            .iter()
            .map(|binding| {
                vk::DescriptorPoolSize {
                    // transform descriptor pool
                    ty: binding.descriptor_type,
                    descriptor_count: images_count,
                }
            })
            .collect();

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DescriptorPoolCreateFlags::empty(),
            max_sets: images_count,
            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
        };

        let descriptor_pool = unsafe {
            self.device
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .expect("Failed to create Descriptor Pool!")
        };

        let layout_create_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            flags: vk::DescriptorSetLayoutCreateFlags::empty(),
            binding_count: bindings.len() as u32,
            p_bindings: bindings.as_ptr(),
            ..Default::default()
        };
        let layouts = unsafe {
            vec![self
                .device
                .create_descriptor_set_layout(&layout_create_info, None)
                .expect("Failed to create Descriptor Set Layout!")]
        };

        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            descriptor_pool,
            descriptor_set_count: 1 as u32,
            p_set_layouts: layouts.as_ptr(),
            ..Default::default()
        };

        let descriptor_sets = unsafe {
            self.device
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .expect("Failed to allocate descriptor sets!")
        };

        for descriptor in descriptor_write_sets.iter_mut() {
            descriptor.dst_set = descriptor_sets[0]
        }

        unsafe {
            self.device
                .update_descriptor_sets(&descriptor_write_sets, &[]);
        }

        (layouts[0], descriptor_sets[0], descriptor_pool)
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            self.wait_idle();
            self.surface
                .surface_loader
                .destroy_surface(self.surface.surface, None);

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
