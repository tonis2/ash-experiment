use ash::{
    extensions::khr::{Surface, Swapchain},
    version::{InstanceV1_0},
    vk, Device, Instance,
};


use std::collections::HashSet;
use crate::modules::queue::{Queue,QueueFamilyIndices};
use std::ptr;


pub fn create_device(
    queue_family: QueueFamilyIndices,
    instance: &Instance,
    pdevice: vk::PhysicalDevice,
) -> (Device, Queue) {
    let device_extension_names_raw = [Swapchain::name().as_ptr()];
    let features = vk::PhysicalDeviceFeatures {
        shader_clip_distance: 1,
        ..Default::default()
    };

    let mut queue_create_infos = vec![];

    let mut unique_queue_families = HashSet::new();
    unique_queue_families.insert(queue_family.graphics_family.unwrap());
    unique_queue_families.insert(queue_family.present_family.unwrap());
    let queue_priorities = [1.0_f32];

    for &queue_family in unique_queue_families.iter() {
        let queue_create_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index: queue_family,
            p_queue_priorities: queue_priorities.as_ptr(),
            queue_count: queue_priorities.len() as u32,
        };
        queue_create_infos.push(queue_create_info);
    }

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_create_infos)
        .enabled_extension_names(&device_extension_names_raw)
        .enabled_features(&features);

    unsafe {
        let device: Device = instance
            .create_device(pdevice, &device_create_info, None)
            .unwrap();

        let queue = Queue::new(&device, queue_family);

        (device, queue)
    }
}

pub fn create_physical_device(
    instance: &Instance,
    surface: &Surface,
    surface_khr: vk::SurfaceKHR,
) -> (vk::PhysicalDevice, QueueFamilyIndices) {
    unsafe {
        let pdevices = instance
            .enumerate_physical_devices()
            .expect("Physical device error");
        let pdevice = pdevices
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
                            Some(*pdevice)
                        } else {
                            None
                        }
                    })
                    .nth(0)
            })
            .filter_map(|v| v)
            .nth(0)
            .expect("Couldn't find suitable device.");

        let queue_families = instance.get_physical_device_queue_family_properties(pdevice);

        let mut queue_family_indices = QueueFamilyIndices::new();

        let mut index = 0;

        for queue_family in queue_families.iter() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                queue_family_indices.graphics_family = Some(index);
            }

            let is_present_support =
                surface.get_physical_device_surface_support(pdevice, index as u32, surface_khr);
            if queue_family.queue_count > 0 && is_present_support.unwrap() {
                queue_family_indices.present_family = Some(index);
            }

            if queue_family_indices.is_complete() {
                break;
            }

            index += 1;
        }

        (pdevice, queue_family_indices)
    }
}
