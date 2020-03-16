use std::path::Path;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub a_pos: [f32; 3],
    pub a_col: [f32; 4],
    pub a_uv: [f32; 2],
    pub a_norm: [f32; 3],
    pub a_joint_indices: [f32; 4],
    pub a_joint_weights: [f32; 4],
}

#[derive(Debug, Clone)]
pub struct Node {
    pub parent: Option<usize>, //Parent Index
    pub children: Vec<usize>,  //Children Indices
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}
#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub struct GLTFModel {
    pub meshes: Vec<Mesh>,
    pub nodes: Vec<Node>,
}

impl GLTFModel {
    pub fn create_from(file_path: &Path) -> Self {
        let (gltf_model, buffers, _) = gltf::import(file_path).unwrap();

        let mut nodes: Vec<Node> = Vec::new();
        let mut meshes: Vec<Mesh> = Vec::new();

        //Store Nodes
        for node in gltf_model.nodes() {
            let children_indices = node
                .children()
                .map(|child| child.index())
                .collect::<Vec<usize>>();

            let (translation, rotation, scale) = node.transform().decomposed();

            let mut parent = None;

            //If we encounter ourselves (node) when searching children, we've found our parent
            for potential_parent in gltf_model.nodes() {
                if potential_parent
                    .children()
                    .find(|child| child.index() == node.index())
                    .is_some()
                {
                    parent = Some(potential_parent.index());
                }
            }

            nodes.push(Node {
                parent: parent,
                children: children_indices,
                translation: translation,
                rotation: rotation,
                scale: scale,
            });

            let parent_index = match nodes[nodes.len() - 1].parent {
                Some(index) => index.to_string(),
                None => "N/A".to_string(),
            };

            println!(
                "INDEX: {},\tPARENT: {},\tCHILDREN {:?}",
                node.index(),
                parent_index,
                nodes[nodes.len() - 1].children
            );

            let has_mesh = node.mesh().is_some();
            let is_skinned = node.skin().is_some();

            //If there is mesh inside node
            if let Some(gltf_mesh) = node.mesh() {
                for primitive in gltf_mesh.primitives() {
                    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                    let mut vertices = Vec::new();
                    let indices: Vec<u32> = reader
                        .read_indices()
                        .map(|read_indices| read_indices.into_u32().collect())
                        .expect("Failed to read indices");

                    let pos_iter = reader.read_positions().unwrap();
                    //TODO: Better error handling if no positions (return Err("Mesh requires positions"))

                    //Normals
                    let mut norm_iter = reader.read_normals();

                    //Optional Colors
                    let mut col_iter = match reader.read_colors(0) {
                        Some(col_iter) => Some(col_iter.into_rgba_f32()),
                        None => None,
                    };

                    //Optional UVs
                    let mut uv_iter = match reader.read_tex_coords(0) {
                        Some(uv_iter) => Some(uv_iter.into_f32()),
                        None => {
                            println!("Warning: Mesh is missing UVs");
                            None
                        }
                    };

                    //if skinned, we need to get the JOINTS_0 and WEIGHTS_0 attributes
                    let mut joints_iter = match reader.read_joints(0) {
                        Some(joints_iter) => Some(joints_iter.into_u16()),
                        None => {
                            println!("NO JOINTS");
                            None
                        }
                    };

                    let mut weights_iter = match reader.read_weights(0) {
                        Some(weights_iter) => Some(weights_iter.into_f32()),
                        None => {
                            println!("NO WEIGHTS");
                            None
                        }
                    };

                    //Iterate over our positions
                    for pos in pos_iter {
                        let col = match &mut col_iter {
                            Some(col_iter) => match col_iter.next() {
                                Some(col) => col,
                                None => [0., 0., 0., 1.0],
                            },
                            None => [0., 0., 0., 1.0],
                        };

                        let uv = match &mut uv_iter {
                            Some(uv_iter) => match uv_iter.next() {
                                Some(uv) => uv,
                                None => [0.0, 0.0],
                            },
                            None => [0.0, 0.0],
                        };

                        let norm = match &mut norm_iter {
                            Some(norm_iter) => match norm_iter.next() {
                                Some(norm) => norm,
                                None => [0.0, 0.0, 0.0],
                            },
                            None => [0.0, 0.0, 0.0],
                        };

                        let joint_indices = match &mut joints_iter {
                            Some(joints_iter) => match joints_iter.next() {
                                Some(joint_indices) => [
                                    joint_indices[0] as f32,
                                    joint_indices[1] as f32,
                                    joint_indices[2] as f32,
                                    joint_indices[3] as f32,
                                ],
                                None => [0., 0., 0., 0.],
                            },
                            None => [0., 0., 0., 0.],
                        };

                        let joint_weights = match &mut weights_iter {
                            Some(weights_iter) => match weights_iter.next() {
                                Some(joint_weights) => joint_weights,
                                None => [0.0, 0.0, 0.0, 0.0],
                            },
                            None => [0.0, 0.0, 0.0, 0.0],
                        };

                        vertices.push(Vertex {
                            a_pos: pos,
                            a_col: col,
                            a_uv: uv,
                            a_norm: norm,
                            a_joint_indices: joint_indices,
                            a_joint_weights: joint_weights,
                        });
                    }

                    meshes.push(Mesh { vertices, indices });
                }
            }
        }
        GLTFModel { nodes, meshes }
    }
}
