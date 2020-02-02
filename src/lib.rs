mod utility;
mod modules;

use ash::util::*;
use ash::vk;

pub struct VulkanBase {
    device: vk::Device,
}