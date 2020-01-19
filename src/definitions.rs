use ash::vk;
use std::os::raw::c_char;

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;
pub const IS_PAINT_FPS_COUNTER: bool = false;

pub struct DeviceExtension {
    pub names: [&'static str; 1],
    //    pub raw_names: [*const i8; 1],
}

impl DeviceExtension {
    pub fn get_extensions_raw_names(&self) -> [*const c_char; 1] {
        [
            // currently just enable the Swapchain extension.
            ash::extensions::khr::Swapchain::name().as_ptr(),
        ]
    }
}

pub struct ValidationInfo {
    pub is_enable: bool,
    pub required_validation_layers: [&'static str; 1],
}

pub struct VulkanInfo {
    pub version: [i32; 3],
    pub engine_version: [i32; 3],
    pub api_version: [i32; 3],
    pub app_name: &'static str,
    pub device_extension: DeviceExtension,
    pub validation_info: ValidationInfo,
}

impl Default for VulkanInfo {
    fn default() -> VulkanInfo {
        VulkanInfo {
            version: [1, 0, 0],
            engine_version: [1, 0, 0],
            api_version: [1, 0, 92],
            app_name: "app",
            device_extension: DeviceExtension {
                names: ["VK_KHR_swapchain"],
            },
            validation_info: ValidationInfo {
                is_enable: true,
                required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
            },
        }
    }
}

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

pub struct SurfaceStuff {
    pub surface_loader: ash::extensions::khr::Surface,
    pub surface: vk::SurfaceKHR,

    pub screen_width: u32,
    pub screen_height: u32,
}
