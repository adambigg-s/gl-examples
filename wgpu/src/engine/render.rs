use std::collections;

use image::GenericImageView;
use wgpu::util::{self, DeviceExt};

#[derive(bon::Builder, Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub binding: wgpu::BindGroup,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn new(
        path: &'static str,
        label: Option<&'static str>,
        context: &RenderContext,
    ) -> anyhow::Result<Self> {
        let image = image::open(path)?.flipv();
        let rgba = image.to_rgba8();
        let (width, height) = image.dimensions();
        let size = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };

        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        context.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = context.device.create_sampler(&wgpu::SamplerDescriptor::default());
        let binding_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let binding = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout: &binding_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Ok(Self { texture, view, binding, sampler })
    }

    pub fn depth_texture(context: &RenderContext) -> Self {
        let size = wgpu::Extent3d {
            width: context.config.width.max(1),
            height: context.config.height.max(1),
            depth_or_array_layers: 1,
        };
        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = context.device.create_sampler(&wgpu::SamplerDescriptor::default());
        let binding_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Depth texture"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let binding = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Depth texture"),
            layout: &binding_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self { texture, view, sampler, binding }
    }
}

pub trait GpuVertex {
    fn descriptor() -> wgpu::VertexBufferLayout<'static>;
}

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
    pub texture: Option<Texture>,
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
    pub global_bindings: Option<Uniform>,
    pub depth_buffer: Texture,
    pub pipelines: collections::HashMap<u32, wgpu::RenderPipeline>,
    pub objects: collections::HashMap<u32, Renderable>,
    pub commands: Vec<RenderCommand>,
}

impl Renderer {
    pub fn new(context: &RenderContext) -> Self {
        Self {
            global_bindings: None,
            depth_buffer: Texture::depth_texture(context),
            pipelines: collections::HashMap::new(),
            objects: collections::HashMap::new(),
            commands: Vec::new(),
        }
    }

    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass) {
        if let Some(bind_group) = &self.global_bindings {
            render_pass.set_bind_group(0, &bind_group.binding, &[]);
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
                if uniform.location == 0 || uniform.location == 1 {
                    log::error!("Groups 0 & 1 are reserved for camera and texture bindings");
                }
                render_pass.set_bind_group(uniform.location, &uniform.binding, &[]);
            }
            if let Some(tex) = &obj.texture {
                render_pass.set_bind_group(1, &tex.binding, &[]);
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
