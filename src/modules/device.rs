use ash::{
    extensions::khr::{Surface, Swapchain},
    version::{DeviceV1_0, InstanceV1_0},
    vk, Device, Instance,
};

use crate::definitions::MAX_FRAMES_IN_FLIGHT;
use std::collections::HashSet;

use std::ptr;


pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn new() -> QueueFamilyIndices {
        QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}
pub struct Queue {
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub queue_family_indices: QueueFamilyIndices,
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub inflight_fences: Vec<vk::Fence>,
    pub current_frame: usize,
}

impl Queue {
    pub fn new<D: DeviceV1_0>(device: &D, queue_family_indices: QueueFamilyIndices) -> Self {
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SemaphoreCreateFlags::empty(),
        };

        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FenceCreateFlags::SIGNALED,
        };

        let mut image_available_semaphores = vec![];
        let mut render_finished_semaphores = vec![];
        let mut inflight_fences = vec![];

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                let image_available_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore Object!");
                let render_finished_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore Object!");
                let inflight_fence = device
                    .create_fence(&fence_create_info, None)
                    .expect("Failed to create Fence Object!");

                image_available_semaphores.push(image_available_semaphore);
                render_finished_semaphores.push(render_finished_semaphore);
                inflight_fences.push(inflight_fence);
            }
        }
        unsafe {
            Self {
                graphics_queue: device.get_device_queue(queue_family_indices.graphics_family.unwrap(), 0),
                present_queue: device.get_device_queue(queue_family_indices.present_family.unwrap(), 0),
                queue_family_indices,
                image_available_semaphores,
                render_finished_semaphores,
                inflight_fences,
                current_frame: 0,
            }
        }
    }

    pub fn destroy(&self, device: &ash::Device) {
        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                device.destroy_semaphore(self.image_available_semaphores[i], None);
                device.destroy_semaphore(self.render_finished_semaphores[i], None);
                device.destroy_fence(self.inflight_fences[i], None);
            }
        }
    }
}

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
    let priorities = [1.0];

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
        let mut unique_queue_families: HashSet<u32> = HashSet::new();
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

        let queue_families =
        unsafe { instance.get_physical_device_queue_family_properties(pdevice) };

        let mut queue_family_indices = QueueFamilyIndices::new();

        let mut index = 0;

        for queue_family in queue_families.iter() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                queue_family_indices.graphics_family = Some(index);
            }
    
            let is_present_support = unsafe {
                surface
                    .get_physical_device_surface_support(
                        pdevice,
                        index as u32,
                        surface_khr,
                    )
            };
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
