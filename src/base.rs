use crate::modules::{
    debug::create_debugger,
    device::{self, Queue},
    frame::Frame,
    surface,
    swapchain::Swapchain,
};

use ash::{
    extensions::khr::Surface,
    version::{DeviceV1_0, InstanceV1_0},
    vk, Device, Entry, Instance,
};

use crate::definitions::MAX_FRAMES_IN_FLIGHT;
use crate::utility::{find_memorytype_index, Buffer};

use std::ptr;
use winit::window::Window;

#[allow(dead_code)]
pub struct VkInstance {
    entry: Entry,
    debugger: vk::DebugReportCallbackEXT,
    pub instance: Instance,
    pub surface: Surface,
    pub surface_khr: vk::SurfaceKHR,
    pub physical_device: vk::PhysicalDevice,
    pub device: Device,
    pub queue: Queue,
}

impl VkInstance {
    pub fn new(window: &Window) -> Self {
        unsafe {
            let (entry, instance) = crate::modules::create_entry();
            let debugger = create_debugger(&entry, &instance);
            let surface = Surface::new(&entry, &instance);
            let surface_khr = surface::create_surface(&entry, &instance, &window).unwrap();

            let (pdevice, queue_family) =
                device::create_physical_device(&instance, &surface, surface_khr);
            let (device, queue) = device::create_device(queue_family, &instance, pdevice);

            Self {
                entry,
                debugger,
                instance,
                surface,
                surface_khr,
                physical_device: pdevice,
                device,
                queue,
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

    pub fn create_command_buffers(
        &self,
        command_pool: vk::CommandPool,
        amount: usize,
    ) -> Vec<vk::CommandBuffer> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: amount as u32,
            command_pool,
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

    pub fn create_command_pool(&self) -> vk::CommandPool {
        unsafe {
            let pool_create_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(self.queue.queue_family_indices.graphics_family.unwrap());

            self.device
                .create_command_pool(&pool_create_info, None)
                .unwrap()
        }
    }

    pub fn build_frame<F: Fn(vk::CommandBuffer, &ash::Device)>(
        &self,
        command_buffers: &Vec<vk::CommandBuffer>,
        frame_buffers: &Vec<vk::Framebuffer>,
        renderpass: &vk::RenderPass,
        render_area: vk::Rect2D,
        swapchain: &Swapchain,
        clear_values: Vec<vk::ClearValue>,
        apply: F,
    ) -> Frame {
        let wait_fences = vec![self.queue.inflight_fences[self.queue.current_frame]];

        let (image_index, is_sub_optimal) = unsafe {
            self.device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");

            swapchain
                .swapchain_loader
                .acquire_next_image(
                    swapchain.swapchain,
                    std::u64::MAX,
                    self.queue.image_available_semaphores[self.queue.current_frame],
                    vk::Fence::null(),
                )
                .expect("Failed to acquire next image.")
        };

        let wait_semaphores = vec![self.queue.image_available_semaphores[self.queue.current_frame]];
        let wait_stages = vec![vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores =
            vec![self.queue.render_finished_semaphores[self.queue.current_frame]];

        let submit_infos = vec![vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &command_buffers[image_index as usize],
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
        }];

        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
        };

        unsafe {
            self.device
                .reset_fences(&wait_fences)
                .expect("Failed to reset Fence!");
            let command_buffer = command_buffers[image_index as usize];

            self.device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording Command Buffer at beginning!");

            let render_pass_begin_info = vk::RenderPassBeginInfo {
                s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
                p_next: ptr::null(),
                render_pass: *renderpass,
                framebuffer: frame_buffers[image_index as usize],
                render_area: render_area,
                clear_value_count: clear_values.len() as u32,
                p_clear_values: clear_values.as_ptr(),
            };

            self.device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            apply(command_buffer, &self.device);

            self.device.cmd_end_render_pass(command_buffer);

            self.device
                .end_command_buffer(command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
        }

        Frame {
            wait_fences,
            wait_semaphores,
            wait_stages,
            signal_semaphores,
            is_sub_optimal,
            image_index,
            submit_infos,
        }
    }

    pub fn render_frame(&mut self, frame: Frame, swapchain: &Swapchain) {
        unsafe {
            self.device
                .queue_submit(
                    self.queue.graphics_queue,
                    &frame.submit_infos,
                    self.queue.inflight_fences[self.queue.current_frame],
                )
                .expect("Failed to execute queue submit.");
        }

        let swapchains = [swapchain.swapchain];

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: frame.signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &frame.image_index,
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

    pub fn update_buffer(&self, from_buffer: Buffer, to_buffer: Buffer) {
        // unsafe {
        //     let data_ptr =
        //         self.device
        //             .map_memory(
        //                 from_buffer.memory,
        //                 0,
        //                 from_buffer.size as u64,
        //                 vk::MemoryMapFlags::empty(),
        //             )
        //             .expect("Failed to Map Memory");

        //     data_ptr.copy_from_nonoverlapping(ubos.as_ptr(), ubos.len());

        //     self.device
        //         .unmap_memory(self.uniform_buffers_memory[current_image]);
        // }
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

impl Drop for VkInstance {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
            self.queue.destroy(&self.device);
            self.device.destroy_device(None);
        }
    }
}
