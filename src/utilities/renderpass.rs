use crate::Context;
use ash::version::DeviceV1_0;
use ash::vk;
use std::sync::Arc;

pub struct Renderpass {
    pass: vk::RenderPass,
    ctx: Arc<Context>,
}

impl Renderpass {
    pub fn new(info: vk::RenderPassCreateInfo, ctx: Arc<Context>) -> Self {
        let pass = unsafe {
            ctx.device
                .create_render_pass(&info, None)
                .expect("Failed to create render pass!")
        };
        Self {
            pass,
            ctx: ctx.clone(),
        }
    }

    pub fn pass(&self) -> vk::RenderPass {
        self.pass
    }
}

impl Drop for Renderpass {
    fn drop(&mut self) {
        unsafe {
            self.ctx.wait_idle();
            self.ctx.device.destroy_render_pass(self.pass, None);
        }
    }
}
