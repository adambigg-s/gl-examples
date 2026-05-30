use std::collections;

use wgpu::util::{self, DeviceExt};

#[derive(bon::Builder, Debug)]
pub struct Uniform {
    pub buffer: Option<wgpu::Buffer>,
    pub binding: wgpu::BindGroup,
    pub location: u32,
}

#[derive(bon::Builder, Debug)]
pub struct Mesh {
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub size: u32,
}

impl Mesh {
    pub fn new(vertices: &[u8], indices: &[u32], device: &wgpu::Device) -> Self {
        let size = indices.len() as u32;
        let vertices = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: vertices,
            usage: wgpu::BufferUsages::VERTEX,
        });
        let indices = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Index buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self { vertices, indices, size }
    }
}

#[derive(bon::Builder, Debug)]
pub struct Renderable {
    pub mesh: Mesh,
    pub uniforms: Vec<Uniform>,
    pub pipeline: u32,
}

#[derive(bon::Builder, Debug)]
pub struct RenderCommand {
    pub object_id: u32,
}

#[derive(bon::Builder, Debug)]
pub struct RenderContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
}

#[derive(bon::Builder, Debug, Default)]
pub struct Renderer {
    pub pipelines: collections::HashMap<u32, wgpu::RenderPipeline>,
    pub objects: collections::HashMap<u32, Renderable>,
    pub commands: Vec<RenderCommand>,
    pub global_bindings: Option<wgpu::BindGroup>,
}

impl Renderer {
    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass) {
        if let Some(bind_group) = &self.global_bindings {
            render_pass.set_bind_group(0, bind_group, &[]);
        }

        for command in &self.commands {
            let Some(obj) = self.objects.get(&command.object_id)
            else {
                log::error!("Render object not found: {}", command.object_id);
                continue;
            };
            let Some(pip) = self.pipelines.get(&obj.pipeline)
            else {
                log::error!("Render pipeline not found: {}", obj.pipeline);
                continue;
            };

            render_pass.set_pipeline(pip);
            render_pass.set_vertex_buffer(0, obj.mesh.vertices.slice(..));
            render_pass.set_index_buffer(obj.mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
            for uniform in &obj.uniforms {
                render_pass.set_bind_group(uniform.location, &uniform.binding, &[]);
            }
            render_pass.draw_indexed(0..obj.mesh.size, 0, 0..1);
        }
        self.commands.clear();
    }

    pub fn register_pipeline(&mut self, pipeline: wgpu::RenderPipeline) -> u32 {
        let tag = self.pipelines.len() as u32;
        self.pipelines.insert(tag, pipeline);
        tag
    }

    pub fn register_object(&mut self, object: Renderable) -> u32 {
        let tag = self.objects.len() as u32;
        self.objects.insert(tag, object);
        tag
    }
}
