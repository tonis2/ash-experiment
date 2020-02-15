use ash::{
    extensions::khr::{Surface, Swapchain},
    version::{DeviceV1_0, InstanceV1_0},
    vk, Device, Instance,
};

pub struct Queue {
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
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
        let present_queue = device.get_device_queue(queue_index as u32, 0);
        let queue = Queue {
            graphics_queue: device.get_device_queue(queue_index as u32, 0),
            present_queue: device.get_device_queue(queue_index as u32, 0),
        };

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
