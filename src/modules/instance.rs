use ash::version::EntryV1_0;
use ash::version::InstanceV1_0;
use ash::vk;
use ash::vk_make_version;
use std::ffi::CString;
use std::ptr;

use crate::definitions::VulkanInfo;
use crate::modules::platforms;

pub fn create_instance(info: &VulkanInfo, entry: &ash::Entry) -> ash::Instance {
    
    //Check validations
    if info.validation_info.is_enable
        && check_validation_layer_support(
            entry,
            &info.validation_info.required_validation_layers.to_vec(),
        ) == false
    {
        panic!("Validation layers requested, but not available!");
    }

    let app_name = CString::new(info.app_name).unwrap();
    let engine_name = CString::new("Vulkan Engine").unwrap();
    let app_info = vk::ApplicationInfo {
        s_type: vk::StructureType::APPLICATION_INFO,
        p_next: ptr::null(),
        p_application_name: app_name.as_ptr(),
        application_version: vk_make_version!(info.version[0], info.version[1], info.version[2]),
        p_engine_name: engine_name.as_ptr(),
        engine_version: vk_make_version!(
            info.engine_version[0],
            info.engine_version[1],
            info.engine_version[2]
        ),
        api_version: vk_make_version!(
            info.api_version[0],
            info.api_version[1],
            info.api_version[2]
        ),
    };

    let extension_names = platforms::required_extension_names();

    let create_info = vk::InstanceCreateInfo {
        s_type: vk::StructureType::INSTANCE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::InstanceCreateFlags::empty(),
        p_application_info: &app_info,
        pp_enabled_layer_names: ptr::null(),
        enabled_layer_count: 0,
        pp_enabled_extension_names: extension_names.as_ptr(),
        enabled_extension_count: extension_names.len() as u32,
    };

    let instance: ash::Instance = unsafe {
        entry
            .create_instance(&create_info, None)
            .expect("Failed to create instance!")
    };

    instance
}

fn check_validation_layer_support(
    entry: &ash::Entry,
    required_validation_layers: &Vec<&str>,
) -> bool {
    // if support validation layer, then return true

    let layer_properties = entry
        .enumerate_instance_layer_properties()
        .expect("Failed to enumerate Instance Layers Properties");

    if layer_properties.len() <= 0 {
        eprintln!("No available layers.");
        return false;
    }

    for required_layer_name in required_validation_layers.iter() {
        let mut is_layer_found = false;

        for layer_property in layer_properties.iter() {
            let test_layer_name = crate::modules::helpers::vk_to_string(&layer_property.layer_name);
            if (*required_layer_name) == test_layer_name {
                is_layer_found = true;
                break;
            }
        }

        if is_layer_found == false {
            return false;
        }
    }

    true
}
