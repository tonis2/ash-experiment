use ash::extensions::ext::DebugReport;
#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
#[cfg(target_os = "windows")]
use ash::extensions::khr::Win32Surface;
#[cfg(target_os = "macos")]
use ash::extensions::mvk::MacOSSurface;
use ash::version::{EntryV1_0, InstanceV1_0};
use ash::{vk, Instance, Entry};
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};

unsafe extern "system" fn vulkan_debug_callback(
    _: vk::DebugReportFlagsEXT,
    _: vk::DebugReportObjectTypeEXT,
    _: u64,
    _: usize,
    _: i32,
    _: *const c_char,
    p_message: *const c_char,
    _: *mut c_void,
) -> u32 {
    println!("{:?}", CStr::from_ptr(p_message));
    vk::FALSE
}

pub unsafe fn create_debugger(
    entry: &Entry,
    instance: &Instance,
) -> vk::DebugReportCallbackEXT {
    let debug_info = vk::DebugReportCallbackCreateInfoEXT::builder()
        .flags(
            vk::DebugReportFlagsEXT::ERROR
                | vk::DebugReportFlagsEXT::WARNING
                | vk::DebugReportFlagsEXT::PERFORMANCE_WARNING,
        )
        .pfn_callback(Some(vulkan_debug_callback));

    let debug_report_loader = DebugReport::new(entry, instance);
    debug_report_loader
        .create_debug_report_callback(&debug_info, None)
        .unwrap()
}
