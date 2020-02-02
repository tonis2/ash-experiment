
use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk;


pub struct Buffer {
    pub size: u32,
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}
