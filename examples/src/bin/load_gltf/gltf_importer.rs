use std::path::Path;
use vulkan::{
    utilities::{Batch, Mesh},
    Buffer,
};

#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

pub struct Node {
    vertex_buffer: Buffer,
    infex_buffer: Buffer,
    indices_len: u32,
    color: [f32; 4],
}

pub struct Importer {
    doc: gltf::Document,
    buffers: Vec<gltf::buffer::Data>,
    images: Vec<gltf::image::Data>,
}

impl Importer {
    pub fn load<P: AsRef<Path>>(path: P) -> Importer {
        let (doc, buffers, images) = gltf::import(path).expect("Failed to load gltf file");
        Importer {
            doc,
            buffers,
            images,
        }
    }

    pub fn build(&self) -> Batch<Vertex> {
        println!("Loading meshes: {}", self.doc.meshes().len());
        let mut batch = Batch::<Vertex>::new();

        for mesh in self.doc.meshes() {
            let mut mesh_data = Mesh::<Vertex>::default();

            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&self.buffers[buffer.index()]));
                let indices_data: Option<Vec<u32>> = reader
                    .read_indices()
                    .map(|read_indices| read_indices.into_u32().collect());

                if indices_data.is_some() {
                    mesh_data.indices = indices_data.unwrap();
                }

                for position in reader.read_positions().unwrap() {
                    mesh_data.vertices.push(Vertex {
                        position,
                        color: [1.0, 1.0, 1.0],
                    });
                    batch.add(&mut mesh_data);
                }
            }
        }
        batch
    }
}
