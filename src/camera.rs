use crate::{mesh, renderer};

const MOVE_SPEED: f32 = 0.05;
const LOOK_SPEED: f32 = 0.05;

#[derive(Default)]
pub struct Camera {
    pub transform: mesh::Transform,
}

impl Camera {
    pub fn get_view(&self) -> glam::Mat4 {
        self.transform.to_matrix().inverse()
    }

    pub fn get_proj(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh_gl(
            renderer::FOV_DEGREES.to_radians(),
            renderer::WINDOW_WIDTH as f32 / renderer::WINDOW_HEIGHT as f32,
            renderer::NEAR_PLANE,
            renderer::FAR_PLANE,
        )
    }

    pub fn update_inputs(&mut self, window: &glfw::PWindow) {
        let mut translation = glam::IVec3::ZERO;
        let mut rotation = glam::IVec2::ZERO;

        if window.get_key(glfw::Key::W) == glfw::Action::Press {
            translation.z -= 1;
        }
        if window.get_key(glfw::Key::S) == glfw::Action::Press {
            translation.z += 1;
        }
        if window.get_key(glfw::Key::A) == glfw::Action::Press {
            translation.x -= 1;
        }
        if window.get_key(glfw::Key::D) == glfw::Action::Press {
            translation.x += 1;
        }
        if window.get_key(glfw::Key::Up) == glfw::Action::Press {
            rotation.x += 1;
        }
        if window.get_key(glfw::Key::Down) == glfw::Action::Press {
            rotation.x -= 1;
        }
        if window.get_key(glfw::Key::Left) == glfw::Action::Press {
            rotation.y += 1;
        }
        if window.get_key(glfw::Key::Right) == glfw::Action::Press {
            rotation.y -= 1;
        }

        let rotation = rotation.as_vec2() * LOOK_SPEED;
        let yaw_quat = glam::Quat::from_rotation_y(rotation.y);
        let pitch_quat = glam::Quat::from_rotation_x(rotation.x);
        self.transform.rotation = yaw_quat * self.transform.rotation * pitch_quat;

        let translation = translation.as_vec3().normalize_or_zero() * MOVE_SPEED;
        let fvec = self.transform.rotation * glam::Vec3::Z;
        let rvec = self.transform.rotation * glam::Vec3::X;
        let uvec = glam::Vec3::Y;
        self.transform.position += fvec * translation.z + rvec * translation.x + uvec * translation.y;
    }
}
