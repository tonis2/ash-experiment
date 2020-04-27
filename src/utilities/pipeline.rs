use crate::Context;
use ash::version::DeviceV1_0;
use ash::vk;
use std::sync::Arc;


pub struct Pipeline {
    layout: vk::PipelineLayout,
    pipelines: Vec<vk::Pipeline>,
    ctx: Arc<Context>,
}

impl Pipeline {
    pub fn new(
        layout: vk::PipelineLayoutCreateInfo,
        info: vk::GraphicsPipelineCreateInfoBuilder,
        ctx: Arc<Context>,
    ) -> Self {
        let layout = unsafe { ctx.device.create_pipeline_layout(&layout, None).unwrap() };

        let pipelines = unsafe {
            ctx.device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[info.layout(layout).build()],
                    None,
                )
                .expect("Unable to create graphics pipeline")
        };

        Self {
            layout,
            pipelines,
            ctx: ctx.clone(),
        }
    }

    pub fn default(&self) -> vk::Pipeline {
        self.pipelines[0]
    }

    pub fn get_pipeline(&self, index: u32) -> vk::Pipeline {
        self.pipelines[index as usize]
    }

    pub fn layout(&self) -> vk::PipelineLayout {
        self.layout
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.ctx.wait_idle();
            self.ctx.device.destroy_pipeline_layout(self.layout, None);
            for pipe in &self.pipelines {
                self.ctx.device.destroy_pipeline(*pipe, None);
            }
        }
    }
}
