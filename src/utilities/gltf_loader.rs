// use std::path::Path;

// #[derive(Debug, Clone, Copy)]
// #[repr(C)]
// pub struct Vertex {
//     pub a_pos: [f32; 3],
//     pub a_col: [f32; 4],
//     pub a_uv: [f32; 2],
//     pub a_norm: [f32; 3],
//     pub a_joint_indices: [f32; 4],
//     pub a_joint_weights: [f32; 4],
// }

// #[derive(Debug, Clone)]
// pub struct Node {
//     pub parent: Option<usize>, //Parent Index
//     pub children: Vec<usize>,  //Children Indices
//     pub translation: [f32; 3],
//     pub rotation: [f32; 4],
//     pub scale: [f32; 3],
// }
// #[derive(Debug, Clone)]
// pub struct Mesh {
//     pub vertices: Vec<Vertex>,
//     pub indices: Vec<u32>,
// }

// pub struct GLTFModel {
//     pub meshes: Vec<Mesh>,
//     pub nodes: Vec<Node>,
// }

// impl GLTFModel {
//     pub fn create_from(file_path: &Path) -> Self {
//         let (gltf, buffers, asset_textures) =
//             gltf::import(&file_path).expect("Couldn't import file!");

//         let mut nodes: Vec<Node> = Vec::new();
//         let mut meshes: Vec<Mesh> = Vec::new();

    

//         //Store Nodes
//         for node in gltf.nodes() {
//             let children_indices = node
//                 .children()
//                 .map(|child| child.index())
//                 .collect::<Vec<usize>>();

//             let (translation, rotation, scale) = node.transform().decomposed();

//             let mut parent = None;

//             //If we encounter ourselves (node) when searching children, we've found our parent
//             for potential_parent in gltf.nodes() {
//                 if potential_parent
//                     .children()
//                     .find(|child| child.index() == node.index())
//                     .is_some()
//                 {
//                     parent = Some(potential_parent.index());
//                 }
//             }

//             nodes.push(Node {
//                 parent: parent,
//                 children: children_indices,
//                 translation: translation,
//                 rotation: rotation,
//                 scale: scale,
//             });

//             let parent_index = match nodes[nodes.len() - 1].parent {
//                 Some(index) => index.to_string(),
//                 None => "N/A".to_string(),
//             };

//             println!(
//                 "INDEX: {},\tPARENT: {},\tCHILDREN {:?}",
//                 node.index(),
//                 parent_index,
//                 nodes[nodes.len() - 1].children
//             );

//             let has_mesh = node.mesh().is_some();
//             let is_skinned = node.skin().is_some();
//         }
//         GLTFModel { nodes, meshes }
//     }
// }
