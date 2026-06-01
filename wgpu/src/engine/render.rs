use std::collections;

use wgpu::util::{self, DeviceExt};

use crate::engine::texture;

pub trait GpuVertex {
    fn descriptor() -> wgpu::VertexBufferLayout<'static>;
}

pub trait CameraUniform {
    fn view_proj_matrix(&self) -> glam::Mat4;
}

#[derive(bon::Builder, Debug)]
pub struct Uniform {
    pub buffer: Option<wgpu::Buffer>,
    pub binding: wgpu::BindGroup,
    pub group: u32,
}

impl Uniform {
    pub fn new() -> Self {
        todo!()
    }
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
    pub texture: Option<texture::Texture>,
    pub pipeline: u32,
}

#[derive(bon::Builder, Debug)]
pub struct RenderCommand {
    pub id: u32,
}

impl RenderCommand {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

#[derive(bon::Builder, Debug)]
pub struct RenderContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
}

#[derive(bon::Builder, Debug)]
pub struct Renderer {
    pub global_bindings: Vec<Uniform>,
    pub depth_buffer: texture::Texture,
    pub pipelines: collections::HashMap<u32, wgpu::RenderPipeline>,
    pub objects: collections::HashMap<u32, Renderable>,
    pub commands: Vec<RenderCommand>,
}

impl Renderer {
    pub fn new(context: &RenderContext) -> Self {
        Self {
            global_bindings: Vec::new(),
            depth_buffer: texture::Texture::depth_texture(context),
            pipelines: collections::HashMap::new(),
            objects: collections::HashMap::new(),
            commands: Vec::new(),
        }
    }

    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass) {
        for uniform in &self.global_bindings {
            render_pass.set_bind_group(uniform.group, &uniform.binding, &[]);
        }

        for command in &self.commands {
            let Some(obj) = self.objects.get(&command.id)
            else {
                log::error!("Render object not found: {}", command.id);
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
                render_pass.set_bind_group(uniform.group, &uniform.binding, &[]);
            }
            if let Some(tex) = &obj.texture {
                render_pass.set_bind_group(2, &tex.binding, &[]);
            }
            render_pass.draw_indexed(0..obj.mesh.size, 0, 0..1);
        }
        self.commands.clear();
    }

    pub fn queue_command(&mut self, command: RenderCommand) {
        self.commands.push(command);
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
