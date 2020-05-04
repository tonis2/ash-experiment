use crate::Context;
use ash::version::DeviceV1_0;
use ash::vk;
use std::sync::Arc;

pub struct Pipeline {
    layouts: Vec<vk::PipelineLayout>,
    pipelines: Vec<vk::Pipeline>,
    ctx: Arc<Context>,
}

impl Pipeline {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self {
            layouts: Vec::new(),
            pipelines: Vec::new(),
            ctx: ctx.clone(),
        }
    }

    pub fn add_layout(&mut self, layout: vk::PipelineLayoutCreateInfo) {
        self.layouts.push(unsafe {
            self.ctx
                .device
                .create_pipeline_layout(&layout, None)
                .unwrap()
        });
    }

    pub fn add_pipeline(&mut self, info: vk::GraphicsPipelineCreateInfo) {
        let pipeline = unsafe {
            self.ctx
                .device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)
                .expect("Unable to create graphics pipeline")
        };
        self.pipelines.push(pipeline[0]);
    }

    pub fn add_compute(&mut self, info: vk::ComputePipelineCreateInfo) {
        let pipeline = unsafe {
            self.ctx
                .device
                .create_compute_pipelines(vk::PipelineCache::null(), &[info], None)
                .expect("Unable to create graphics pipeline")
        };
        self.pipelines.push(pipeline[0]);
    }

    pub fn default(&self) -> vk::Pipeline {
        self.pipelines[0]
    }

    pub fn pipeline(&self, index: usize) -> vk::Pipeline {
        self.pipelines[index]
    }

    pub fn layout(&self, index: usize) -> vk::PipelineLayout {
        self.layouts[index]
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.ctx.wait_idle();
            for layout in &self.layouts {
                self.ctx.device.destroy_pipeline_layout(*layout, None);
            }
            for pipe in &self.pipelines {
                self.ctx.device.destroy_pipeline(*pipe, None);
            }
        }
    }
}
