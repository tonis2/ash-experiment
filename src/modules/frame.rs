use ash::vk;

pub struct Frame {
    pub wait_fences: Vec<vk::Fence>,
    pub signal_semaphores: Vec<vk::Semaphore>,
    pub wait_semaphores: Vec<vk::Semaphore>,
    pub image_index: u32,
    pub is_sub_optimal: bool,
    pub wait_stages: Vec<vk::PipelineStageFlags>,
    pub submit_infos: Vec<vk::SubmitInfo>
}