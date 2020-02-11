pub mod buffer;
pub mod helpers;
pub mod shader;

pub use buffer::Buffer;

use crate::modules::surface::extension_names;
use ash::version::{EntryV1_0, InstanceV1_0};
use ash::{vk, Entry, Instance};
use std::ffi::CString;

pub unsafe fn create_entry(window: &winit::window::Window) -> (Entry, Instance) {
    let entry = Entry::new().unwrap();
    let app_name = CString::new("VulkanTriangle").unwrap();

    let layer_names = [CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()];
    let layers_names_raw: Vec<*const i8> = layer_names
        .iter()
        .map(|raw_name| raw_name.as_ptr())
        .collect();

    let extension_names_raw = extension_names();

    let appinfo = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(0)
        .engine_name(&app_name)
        .engine_version(0)
        .api_version(vk::make_version(1, 0, 0));

    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&appinfo)
        .enabled_layer_names(&layers_names_raw)
        .enabled_extension_names(&extension_names_raw);

    let instance: Instance = entry
        .create_instance(&create_info, None)
        .expect("Instance creation error");

    (entry, instance)
}
