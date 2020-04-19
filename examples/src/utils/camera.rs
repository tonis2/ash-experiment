use super::Event;
use cgmath::{Deg, Matrix4, Vector3, Vector4};
//From this ThinMatrix tutorial https://www.youtube.com/watch?v=PoxDDZmctnU
#[derive(Clone, Debug, Copy)]
pub struct Camera {
    eye: cgmath::Point3<f32>,
    focus: cgmath::Point3<f32>,
    zoom: f32,
    aspect: f32,
    yaw: f32,   //Rotation around x-axis, left-to-right
    pitch: f32, //Rotation around y-axis, top-down

    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Camera {
    pub fn new(focus: cgmath::Point3<f32>, zoom: f32, aspect: f32) -> Self {
        let mut camera = Camera {
            eye: cgmath::Point3::new(0.0, 0.0, zoom),
            focus,
            zoom,
            aspect,
            yaw: -90.0,
            pitch: 50.0,
            min_zoom: 0.0,
            max_zoom: 100.0
        };
        camera.calculate_eye_position();
        camera
    }

    fn calculate_eye_position(&mut self) {
        //Calculate the distances from camera to our focus point
        let horizontal_distance = self.zoom * self.pitch.to_radians().cos();
        let vertical_distance = self.zoom * self.pitch.to_radians().sin();
        let offset_x: f32 = horizontal_distance * self.yaw.to_radians().sin();
        let offset_z: f32 = horizontal_distance * self.yaw.to_radians().cos();
        self.eye = cgmath::Point3::new(
            self.focus.x - offset_x,
            self.focus.y + vertical_distance,
            self.focus.z - offset_z,
        );
    }

    pub fn add_zoom(&mut self, value: f32) {
        //Min max zoom amount
        if (self.zoom + value).abs() > self.min_zoom && (self.zoom + value).abs() < self.max_zoom {
            self.zoom += value;
        }
    }

    pub fn rotate_x(&mut self, deg: f32) {
        self.yaw += deg;
    }

    pub fn rotate_y(&mut self, deg: f32) {
        const MAX_TURN: f32 = 69.0;
        self.pitch -= deg;

        if self.pitch > MAX_TURN {
            self.pitch = MAX_TURN;
        }

        if self.pitch < -MAX_TURN {
            self.pitch = -MAX_TURN;
        }
    }

    pub fn handle_events(&mut self, events: &Event) {
        //Rotate left, right
        if events.keyboard.key_pressed("d") {
            self.rotate_x(1.0);
        }

        if events.keyboard.key_pressed("a") {
            self.rotate_x(-1.0);
        }

        //Mouse scroll zoom
        self.add_zoom(-events.mouse.scroll());

        //Zoom in out
        if events.keyboard.key_pressed("w") {
            self.add_zoom(1.0);
        }

        if events.keyboard.key_pressed("s") {
            self.add_zoom(-1.0);
        }

        //Mouse movement
        if events.mouse.on_right_click() {
            let mouse_pos = events.mouse.position_delta();
            self.rotate_x(mouse_pos.x);
            self.rotate_y(mouse_pos.y);
        }
        self.calculate_eye_position();
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct CameraRaw {
    pub position: Vector4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

impl Camera {
    pub fn raw(&self) -> CameraRaw {
        let view: Matrix4<f32> =
            Matrix4::look_at(self.eye, self.focus, Vector3::new(0.0, 1.0, 0.0));
        CameraRaw {
            position: self.eye.to_homogeneous(),
            view,
            proj: {
                let proj = cgmath::perspective(Deg(45.0), self.aspect, 0.1, 70.0);
                super::super::OPENGL_TO_VULKAN_MATRIX * proj
            },
        }
    }
}
