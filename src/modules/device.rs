use ash::{
    extensions::khr::{Surface, Swapchain},
    version::{DeviceV1_0, InstanceV1_0},
    vk, Device, Instance,
};

const MAX_FRAMES_IN_FLIGHT: usize = 2;
use std::ptr;

pub struct Queue {
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    inflight_fences: Vec<vk::Fence>,
}

impl Queue {
    pub fn new<D: DeviceV1_0>(device: &D, queue_index: u32) -> Self {
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
                graphics_queue: device.get_device_queue(queue_index as u32, 0),
                present_queue: device.get_device_queue(queue_index as u32, 0),
                image_available_semaphores,
                render_finished_semaphores,
                inflight_fences,
            }
        }
    }
}

pub fn create_device(
    queue_index: u32,
    instance: &Instance,
    pdevice: vk::PhysicalDevice,
) -> (Device, Queue) {
    let device_extension_names_raw = [Swapchain::name().as_ptr()];
    let features = vk::PhysicalDeviceFeatures {
        shader_clip_distance: 1,
        ..Default::default()
    };
    let priorities = [1.0];

    let queue_info = [vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_index)
        .queue_priorities(&priorities)
        .build()];

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_info)
        .enabled_extension_names(&device_extension_names_raw)
        .enabled_features(&features);

    unsafe {
        let device: Device = instance
            .create_device(pdevice, &device_create_info, None)
            .unwrap();

        let queue = Queue::new(&device, queue_index);

        (device, queue)
    }
}

pub fn create_physical_device(
    instance: &Instance,
    surface: &Surface,
    surface_khr: vk::SurfaceKHR,
) -> (vk::PhysicalDevice, u32) {
    unsafe {
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
        (pdevice, queue_family_index)
    }
}
