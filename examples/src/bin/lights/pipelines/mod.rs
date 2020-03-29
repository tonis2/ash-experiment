pub mod mesh_pipeline;
pub mod shadowmap_pipeline;

use cgmath::{Deg, Matrix4, Point3, Vector3, Vector4};

#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Light {
    pub projection: Matrix4<f32>,
    pub pos: cgmath::Point3<f32>,
    pub color: [f32; 3],
    pub ambient: f32,
    pub specular: f32,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct PushConstantModel {
    pub model: Matrix4<f32>,
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Camera {
    pub position: Vector4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

impl PushConstantModel {
    pub fn new(
        transform: cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Basis3<f32>>,
        color: [f32; 3],
    ) -> Self {
        Self {
            model: transform.into(),
            color: [color[0], color[1], color[2], 1.0],
        }
    }

    pub fn update_transform(
        &mut self,
        transform: cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Basis3<f32>>,
    ) {
        self.model = self.model * cgmath::Matrix4::from(transform);
    }
}

impl Camera {
    pub fn new(aspect: f32, position: cgmath::Point3<f32>) -> Camera {
        Camera {
            position: position.to_homogeneous(),
            view: Matrix4::look_at(
                position,
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
            ),
            proj: {
                let proj = cgmath::perspective(Deg(45.0), aspect, 0.1, 100.0);
                examples::OPENGL_TO_VULKAN_MATRIX * proj
            },
        }
    }
}

impl Light {
    pub fn new(
        pos: cgmath::Point3<f32>,
        aspect: f32,
        color: [f32; 3],
        ambient: f32,
        specular: f32,
    ) -> Light {
        let view = Matrix4::look_at(pos, Point3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 1.0));
        let projection = cgmath::perspective(Deg(45.0), aspect, 0.1, 15.0);

        Light {
            projection: examples::OPENGL_TO_VULKAN_MATRIX * projection * view,
            pos,
            color,
            ambient,
            specular,
        }
    }
}
