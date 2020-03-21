use ash::{
    version::{DeviceV1_0, EntryV1_0, InstanceV1_0},
    vk, Device, Entry, Instance,
};

use super::{debug::Debugger, device, queue::QueueFamilyIndices, surface::VkSurface};
use crate::constants::*;
use crate::utilities::platform::extension_names;
use std::ptr;

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

    pub fn build_command<F: Fn(vk::CommandBuffer, &ash::Device)>(
        &self,
        command_buffer: vk::CommandBuffer,
        render_pass_info: &vk::RenderPassBeginInfo,
        apply: F,
    ) {
        //Build frame buffer

        //build command buffer
        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
        };

        unsafe {
            self.device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording Command Buffer at beginning!");

            self.device.cmd_begin_render_pass(
                command_buffer,
                render_pass_info,
                vk::SubpassContents::INLINE,
            );

            apply(command_buffer, &self.device);

            self.device.cmd_end_render_pass(command_buffer);

            self.device
                .end_command_buffer(command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
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
            self.device.destroy_device(None);
            self._debugger.destroy();
            self.instance.destroy_instance(None);
        }
    }
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
