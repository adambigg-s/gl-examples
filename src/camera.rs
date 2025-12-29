use crate::{mesh, renderer};

#[derive(Default)]
pub struct Camera {
    pub transform: mesh::Transform,
}

impl Camera {
    pub fn new() -> Self {
        Default::default()
    }

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
}
