pub const MAX_FRAMES_IN_FLIGHT: usize = 2;
use ash::vk;

pub struct RenderPassInfo {
  pub render_area: vk::Rect2D,
  pub clear: vk::ClearValue
}


