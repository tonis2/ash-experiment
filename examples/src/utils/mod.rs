mod camera;
pub mod events;
pub mod gltf_importer;

pub use camera::{Camera, CameraRaw};
pub use events::Event;

pub trait MeshTrait<T> {
    fn get_indices(&mut self) -> Vec<u32>;
    fn get_vertices(&mut self) -> Vec<T>;
}

#[derive(Debug, Clone)]
pub struct Mesh<T: Clone> {
    pub vertices: Vec<T>,
    pub indices: Vec<u32>,
}

impl<T: Clone> Default for Mesh<T> {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl<T: Clone> MeshTrait<T> for Mesh<T> {
    fn get_indices(&mut self) -> Vec<u32> {
        self.indices.clone()
    }
    fn get_vertices(&mut self) -> Vec<T> {
        self.vertices.clone()
    }
}

#[derive(Clone)]
pub struct Batch<T: Clone> {
    pub indices: Vec<u32>,
    pub vertices: Vec<T>,
}

impl<T: Clone> Batch<T> {
    pub fn new() -> Self {
        Self {
            indices: Vec::new(),
            vertices: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn add(&mut self, mesh: &mut Mesh<T>) {
        for indice in mesh.get_indices() {
            self.indices
                .push((indice as i64 + self.vertices.len() as i64) as u32);
        }

        self.vertices.extend(mesh.get_vertices());
    }
}
