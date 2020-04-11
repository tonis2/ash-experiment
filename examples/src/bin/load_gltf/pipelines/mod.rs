pub mod mesh_pipeline;

use cgmath::{Deg, Matrix4, Point3, Vector3};

pub use crate::gltf_importer::Vertex;


#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct PushTransform {
    pub transform: cgmath::Matrix4<f32>,
}
#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Light {
    pub position: cgmath::Vector4<f32>,
    pub projection: cgmath::Matrix4<f32>,
    pub color: [f32; 4],
    pub ambient: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Camera {
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

impl Camera {
    pub fn new(aspect: f32, position: Point3<f32>) -> Camera {
        Camera {
            view: Matrix4::look_at(
                position,
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
            ),
            proj: {
                let proj = cgmath::perspective(
                    Deg(45.0),
                    aspect,
                    0.1,
                    30.0,
                );
    
                examples::OPENGL_TO_VULKAN_MATRIX * proj
            },
        }
    }
}

impl Light {
    pub fn new(position: cgmath::Vector4<f32>, color: [f32; 4], ambient: [f32; 4]) -> Self {
        let view = Matrix4::look_at(
            cgmath::Point3::new(position.x, position.y, position.z),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        );

        let projection = examples::OPENGL_TO_VULKAN_MATRIX
            * cgmath::perspective(Deg(45.0), 1.0, 3.0, 30.0)
            * view;

        Self {
            position,
            projection,
            color,
            ambient,
        }
    }
}
