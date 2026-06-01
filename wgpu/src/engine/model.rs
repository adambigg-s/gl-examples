use std::mem;

use crate::engine::render;

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, bon::Builder, Debug, Default, Clone, Copy)]
pub struct ModelVertex {
    pub pos: glam::Vec3,
    pub nor: glam::Vec3,
    pub tex: glam::Vec2,
}

impl render::GpuVertex for ModelVertex {
    fn descriptor() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBS: [wgpu::VertexAttribute; 3] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2];
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}

pub fn load_model_mesh(
    obj: &'static str,
    context: &render::RenderContext,
) -> anyhow::Result<Vec<render::Mesh>> {
    let (models, _) = tobj::load_obj(
        obj,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    )?;

    let mut meshes = Vec::new();
    #[allow(clippy::identity_op)]
    for model in models {
        let mesh = &model.mesh;
        let mut vertices = Vec::new();
        for idx in 0..mesh.positions.len() / 3 {
            #[rustfmt::skip]
            vertices.push(
                ModelVertex::builder()
                    .pos(
                        (
                            mesh.positions[idx * 3 + 0],
                            mesh.positions[idx * 3 + 1],
                            mesh.positions[idx * 3 + 2],
                        )
                            .into(),
                    )
                    .nor(
                        (
                            mesh.normals[idx * 3 + 0],
                            mesh.normals[idx * 3 + 1],
                            mesh.normals[idx * 3 + 2]
                        )
                            .into(),
                    )
                    .tex(
                        (
                            mesh.texcoords[idx * 2 + 0],
                            mesh.texcoords[idx * 2 + 1]
                        )
                            .into()
                    )
                        .build(),
            );
        }
        meshes.push(render::Mesh::new(bytemuck::cast_slice(&vertices), &mesh.indices, &context.device));
    }

    Ok(meshes)
}
