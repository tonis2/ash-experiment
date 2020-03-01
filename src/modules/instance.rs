use ash::{
    version::{DeviceV1_0, EntryV1_0, InstanceV1_0},
    vk, Device, Entry, Instance,
};

use std::ffi::CString;
use std::ptr;
use winit::window::Window;

use super::{
    debug::create_debugger,
    device,
    queue::{Frame, Queue},
    surface::VkSurface,
    swapchain::Swapchain,
};
use crate::constants::*;
use crate::utilities::{buffer::Buffer, platform::extension_names, tools::find_memorytype_index};

pub struct VkInstance {
    _entry: Entry,
    _debugger: vk::DebugReportCallbackEXT,
    pub instance: Instance,
    pub surface: VkSurface,
    pub physical_device: vk::PhysicalDevice,
    pub device: Device,
    pub queue: Queue,
    pub command_pool: vk::CommandPool,
}

impl VkInstance {
    pub fn new(window: &Window) -> VkInstance {
        unsafe {
            let (entry, instance) = create_entry();
            let debugger = create_debugger(&entry, &instance);
            let surface = VkSurface::new(&window, &instance, &entry);
            let physical_device =
                device::pick_physical_device(&instance, &surface, &DEVICE_EXTENSIONS);
            let (device, queue) = device::create_logical_device(
                &instance,
                physical_device,
                &VALIDATION,
                &DEVICE_EXTENSIONS,
                &surface,
            );

            let command_pool = Self::create_command_pool(&device, &queue);

            VkInstance {
                _entry: entry,
                _debugger: debugger,
                instance,
                surface,
                physical_device,
                device,
                queue,
                command_pool,
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

    pub fn create_command_buffers(&self, amount: usize) -> Vec<vk::CommandBuffer> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: amount as u32,
            command_pool: self.command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
        };

        unsafe {
            let command_buffers = self
                .device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate Command Buffers!");
            command_buffers
        }
    }

    pub fn create_command_pool(device: &ash::Device, queue: &Queue) -> vk::CommandPool {
        unsafe {
            let pool_create_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(queue.queue_family_indices.graphics_family.unwrap());

            device.create_command_pool(&pool_create_info, None).unwrap()
        }
    }

    pub fn build_frame<F: Fn(vk::CommandBuffer, &ash::Device)>(
        &self,
        frame: &Frame,
        command_buffers: &Vec<vk::CommandBuffer>,
        frame_buffers: &Vec<vk::Framebuffer>,
        renderpass: &vk::RenderPass,
        render_area: vk::Rect2D,
        clear_values: Vec<vk::ClearValue>,
        apply: F,
    ) {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
        };
        unsafe {
            self.device
                .begin_command_buffer(
                    command_buffers[frame.image_index],
                    &command_buffer_begin_info,
                )
                .expect("Failed to begin recording Command Buffer at beginning!");

            let render_pass_begin_info = vk::RenderPassBeginInfo {
                s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
                p_next: ptr::null(),
                render_pass: *renderpass,
                framebuffer: frame_buffers[frame.image_index],
                render_area: render_area,
                clear_value_count: clear_values.len() as u32,
                p_clear_values: clear_values.as_ptr(),
            };

            self.device.cmd_begin_render_pass(
                command_buffers[frame.image_index],
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            apply(command_buffers[frame.image_index], &self.device);

            self.device
                .cmd_end_render_pass(command_buffers[frame.image_index]);

            self.device
                .end_command_buffer(command_buffers[frame.image_index])
                .expect("Failed to record Command Buffer at Ending!");
        }
    }

    pub fn render_frame(
        &mut self,
        frame: Frame,
        swapchain: &Swapchain,
        command_buffers: &Vec<vk::CommandBuffer>,
    ) {
        let submit_infos = vec![vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: frame.wait_semaphores.len() as u32,
            p_wait_semaphores: frame.wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: frame.wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &command_buffers[frame.image_index as usize],
            signal_semaphore_count: frame.signal_semaphores.len() as u32,
            p_signal_semaphores: frame.signal_semaphores.as_ptr(),
        }];

        unsafe {
            self.device
                .reset_fences(&frame.wait_fences)
                .expect("Failed to reset Fence!");
            self.device
                .queue_submit(
                    self.queue.graphics_queue,
                    &submit_infos,
                    self.queue.inflight_fences[self.queue.current_frame],
                )
                .expect("Failed to execute queue submit.");
        }

        let swapchains = [swapchain.swapchain];
        let image_index = frame.image_index as u32;
        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: frame.signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_index,
            p_results: ptr::null_mut(),
        };

        unsafe {
            swapchain
                .swapchain_loader
                .queue_present(self.queue.present_queue, &present_info)
                .expect("Failed to execute queue present.");
        }

        self.queue.current_frame = (self.queue.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    pub fn create_descriptor_pool(&self, amount: usize) -> vk::DescriptorPool {
        let pool_sizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: amount as u32,
        }];

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DescriptorPoolCreateFlags::empty(),
            max_sets: amount as u32,
            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
        };

        unsafe {
            self.device
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .expect("Failed to create Descriptor Pool!")
        }
    }

    pub fn create_buffer(&self, info: vk::BufferCreateInfo) -> Buffer {
        unsafe {
            let buffer = self.device.create_buffer(&info, None).unwrap();
            let memory_requirements = self.device.get_buffer_memory_requirements(buffer);
            let memory_index = find_memorytype_index(
                &memory_requirements,
                &self.get_physical_device_memory_properties(),
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
            .expect("Unable to find suitable memorytype for the index buffer.");

            let buffer_memory_allocate = vk::MemoryAllocateInfo {
                allocation_size: memory_requirements.size,
                memory_type_index: memory_index,
                ..Default::default()
            };

            let device_memory = self
                .device
                .allocate_memory(&buffer_memory_allocate, None)
                .unwrap();

            Buffer {
                size: 0,
                buffer,
                memory: device_memory,
                memory_requirements,
            }
        }
    }

    pub fn copy_buffer(
        &self,
        device: &ash::Device,
        src_buffer: vk::Buffer,
        dst_buffer: vk::Buffer,
        size: vk::DeviceSize,
    ) {
        let command_buffer = Self::begin_single_time_command(device, self.command_pool);

        let copy_regions = [vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size,
        }];

        unsafe {
            device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &copy_regions);
        }

        Self::end_single_time_command(
            device,
            self.command_pool,
            self.queue.present_queue,
            command_buffer,
        );
    }

    pub fn begin_single_time_command(
        device: &ash::Device,
        command_pool: vk::CommandPool,
    ) -> vk::CommandBuffer {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: 1,
            command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
        };

        let command_buffer = unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate Command Buffers!")
        }[0];

        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        };

        unsafe {
            device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording Command Buffer at beginning!");
        }

        command_buffer
    }

    pub fn end_single_time_command(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
        command_buffer: vk::CommandBuffer,
    ) {
        unsafe {
            device
                .end_command_buffer(command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
        }

        let buffers_to_submit = [command_buffer];

        let sumbit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: 0,
            p_wait_semaphores: ptr::null(),
            p_wait_dst_stage_mask: ptr::null(),
            command_buffer_count: 1,
            p_command_buffers: buffers_to_submit.as_ptr(),
            signal_semaphore_count: 0,
            p_signal_semaphores: ptr::null(),
        }];

        unsafe {
            device
                .queue_submit(submit_queue, &sumbit_infos, vk::Fence::null())
                .expect("Failed to Queue Submit!");
            device
                .queue_wait_idle(submit_queue)
                .expect("Failed to wait Queue idle!");
            device.free_command_buffers(command_pool, &buffers_to_submit);
        }
    }
}

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
