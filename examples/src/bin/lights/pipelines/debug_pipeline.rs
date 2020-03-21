use ash::vk;


#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 3],
    pub tex_coord: [f32; 2],
}