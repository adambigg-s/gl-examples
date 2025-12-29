use crate::renderer;

#[derive(Default)]
pub struct Camera {
    fvec: glam::Vec3,
    rvec: glam::Vec3,
    uvec: glam::Vec3,
    wuvec: glam::Vec3,
    pos: glam::Vec3,
    pitch: f32,
    yaw: f32,
}

impl Camera {
    pub fn new() -> Self {
        let mut out = Camera { wuvec: glam::Vec3::Y, yaw: -90.0, ..Default::default() };
        out.update_vectors();
        out
    }

    pub fn update_vectors(&mut self) {
        let (ys, yc) = self.yaw.to_radians().sin_cos();
        let (ps, pc) = self.pitch.to_radians().sin_cos();
        self.fvec = glam::vec3(yc * pc, ps, ys * pc).normalize();
        self.rvec = self.fvec.cross(self.wuvec).normalize();
        self.uvec = self.rvec.cross(self.fvec).normalize();
    }

    pub fn get_view(&self) -> glam::Mat4 {
        glam::Mat4::look_at_rh(self.pos, self.pos + self.fvec, self.wuvec)
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
