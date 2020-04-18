use cgmath::{Deg, EuclideanSpace, InnerSpace, Matrix4, Vector3};

use examples::events::Event;
use std::mem;
use vulkan::{offset_of, prelude::*};

pub struct SpecializationData {
    pub materials_amount: u32,
    pub textures_amount: u32,
}

impl SpecializationData {
    pub fn specialization_map_entries(&self) -> [vk::SpecializationMapEntry; 2] {
        // Each shader constant of a shader stage corresponds to one map entry
        [
            vk::SpecializationMapEntry {
                constant_id: 0,
                offset: offset_of!(Self, materials_amount) as _,
                size: mem::size_of::<u32>(),
            },
            vk::SpecializationMapEntry {
                constant_id: 1,
                offset: offset_of!(Self, textures_amount) as _,
                size: mem::size_of::<u32>(),
            },
        ]
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct PushTransform {
    pub transform: cgmath::Matrix4<f32>,
}

//From this tutorial https://learnopengl.com/Getting-started/Camera
#[derive(Clone, Debug, Copy)]
pub struct Camera {
    position: cgmath::Point2<f32>,
    zoom: f32,
    aspect: f32,
    yaw: f32,
    pitch: f32,
}

impl Camera {
    pub fn new(position: cgmath::Point2<f32>, zoom: f32, aspect: f32) -> Self {
        Self {
            position,
            zoom,
            aspect,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    pub fn set_zoom(&mut self, value: f32) {
        const MIN_ZOOM: f32 = 8.0;
        const MAX_ZOOM: f32 = 25.0;

        //Min max zoom amount
        if (self.zoom + value).abs() > MIN_ZOOM && (self.zoom + value).abs() < MAX_ZOOM {
            self.zoom += value;
        }
    }

    pub fn handle_events(&mut self, events: &Event) {
        self.set_zoom(events.mouse.scroll());
        const MOVE_AMOUNT: f32 = 0.01;
        if events.keyboard.key_pressed("d") {
            self.position += cgmath::Vector2::new(MOVE_AMOUNT, 0.0);
        }

        if events.keyboard.key_pressed("a") {
            self.position += cgmath::Vector2::new(-MOVE_AMOUNT, 0.0);
        }

        if events.keyboard.key_pressed("w") {
            self.zoom += MOVE_AMOUNT;
        }

        if events.keyboard.key_pressed("s") {
            self.zoom -= MOVE_AMOUNT;
        }

        if events.mouse.on_right_click() {
            let mouse_pos = events.mouse.position_delta();
            self.yaw -= mouse_pos.x * MOVE_AMOUNT;
            self.pitch -= -mouse_pos.y * MOVE_AMOUNT;
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct CameraRaw {
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

impl Camera {
    pub fn raw(&self) -> CameraRaw {
        // let yaw = self.yaw.to_radians();
        // let pitch = self.pitch.to_radians();
        let camera_direction =
            cgmath::Vector3::new(self.yaw, self.pitch, -1.0);

        CameraRaw {
            view: Matrix4::look_at_dir(
                cgmath::Point3::new(self.position.x, self.position.y, self.zoom),
                camera_direction,
                Vector3::new(0.0, 1.0, 0.0),
            ),
            proj: {
                let proj = cgmath::perspective(Deg(45.0), self.aspect, 0.1, 70.0);
                examples::OPENGL_TO_VULKAN_MATRIX * proj
            },
        }
    }
}
