use std::{f32, fmt};

use crate::engine::render;

pub const DEFAULT_RENDER_NEAR: f32 = 0.1;
pub const DEFAULT_RENDER_DISTANCE: f32 = 500.0;
pub const DEFAULT_FOV: f32 = 67.0;

#[derive(bon::Builder, Debug, Default)]
pub struct Camera {
    #[builder(default)]
    pub fvec: glam::Vec3,
    #[builder(default)]
    pub rvec: glam::Vec3,
    #[builder(default)]
    pub uvec: glam::Vec3,
    #[builder(default = glam::Vec3::Y)]
    pub wupvec: glam::Vec3,
    #[builder(default)]
    pub yaw: f32,
    #[builder(default)]
    pub pitch: f32,
    #[builder(default)]
    pub pos: glam::Vec3,
    #[builder(default = DEFAULT_FOV)]
    pub fov: f32,
    #[builder(default = DEFAULT_RENDER_NEAR)]
    pub near: f32,
    #[builder(default = DEFAULT_RENDER_DISTANCE)]
    pub renderdist: f32,
    #[builder(default = 2.0)]
    pub aspect: f32,
}

impl Camera {
    pub fn view(&self) -> glam::Mat4 {
        glam::Mat4::look_to_rh(self.pos, self.fvec, self.wupvec)
    }

    pub fn proj(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh(self.fov.to_radians(), self.aspect, self.near, self.renderdist)
    }

    pub fn update_vectors(&mut self) {
        self.fvec = glam::Mat3::from_rotation_y(self.yaw)
            * glam::Mat3::from_rotation_x(self.pitch)
            * glam::Vec3::NEG_Z;
        self.rvec = self.fvec.cross(self.wupvec);
        self.uvec = self.rvec.cross(self.fvec);

        self.fvec = self.fvec.normalize();
        self.rvec = self.rvec.normalize();
        self.uvec = self.uvec.normalize();
    }

    pub fn update_rotation(&mut self, dp: f32, dy: f32) {
        self.pitch += dp;
        self.yaw += dy;

        self.pitch = self.pitch.clamp(-f32::consts::PI / 2.0 * 0.99, f32::consts::PI / 2.0 * 0.99);
        self.yaw %= f32::consts::TAU;

        self.update_vectors();
    }

    pub fn update_translation(&mut self, dx: f32, dy: f32, dz: f32) {
        self.pos += self.fvec * dz;
        self.pos += self.rvec * dx;
        self.pos += self.wupvec * dy;
    }
}

impl fmt::Display for Camera {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "P: {:.2}", self.pos)?;
        writeln!(fmt, "F: {:.2}", self.fvec)?;
        writeln!(fmt, "R: {:.2}", self.rvec)?;
        writeln!(fmt, "U: {:.2}", self.uvec)?;
        Ok(())
    }
}

impl render::CameraUniform for Camera {
    fn view_proj_matrix(&self) -> glam::Mat4 {
        self.proj() * self.view()
    }
}
