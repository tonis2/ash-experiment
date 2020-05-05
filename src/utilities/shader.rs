use crate::Context;
use ash::util::read_spv;
use ash::version::DeviceV1_0;
use ash::vk;
use std::{default::Default, ffi::CString, path::Path, sync::Arc};

pub struct Shader {
    pub shader_info: vk::PipelineShaderStageCreateInfo,
    shader_module: vk::ShaderModule,
    context: Arc<Context>,
}

impl Shader {
    pub fn new(
        path: &Path,
        stage: vk::ShaderStageFlags,
        entry_name: &CString,
        context: Arc<Context>,
    ) -> Self {
        let shader_module = unsafe {
            context
                .device
                .create_shader_module(
                    &vk::ShaderModuleCreateInfo::builder().code(&load_shader(path)),
                    None,
                )
                .expect("Shader module error")
        };

        Self {
            shader_info: vk::PipelineShaderStageCreateInfo {
                module: shader_module,
                p_name: entry_name.as_ptr(),
                stage: stage,
                ..Default::default()
            },
            shader_module,
            context: context,
        }
    }

    pub fn use_specialization(mut self, info: vk::SpecializationInfo) -> Self {
        self.shader_info.p_specialization_info = &info;
        self
    }

    pub fn info(&self) -> vk::PipelineShaderStageCreateInfo {
        self.shader_info.clone()
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.context
                .device
                .destroy_shader_module(self.shader_module, None);
        }
    }
}

pub fn load_shader(shader_path: &Path) -> Vec<u32> {
    use std::fs::File;
    use std::io::Read;
    let mut shader_data =
        File::open(shader_path).expect(&format!("failed to read shader {:?}", shader_path));
    let mut buffer = Vec::new();
    shader_data
        .read_to_end(&mut buffer)
        .expect("Failed to load shader data");

    read_spv(&mut shader_data).expect("Failed to read vertex shader spv file")
}
