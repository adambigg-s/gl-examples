use std::mem;

use crate::{application, render};

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, bon::Builder, Debug, Default, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    pub fn descriptor() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x3, 1=>Float32x3];
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}

pub struct GameState {}

impl application::Application for GameState {
    fn setup(context: &mut crate::render::RenderContext, renderer: &mut crate::render::Renderer) -> Self {
        let pipeline = {
            let shader = context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

            let render_pipline_layout =
                context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render pipeline layout"),
                    bind_group_layouts: &[],
                    immediate_size: 0,
                });
            context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render pipeline"),
                layout: Some(&render_pipline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[Vertex::descriptor()],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: context.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let pid = renderer.register_pipeline(pipeline);

        let obj1 = render::Renderable {
            mesh: render::Mesh::new(
                bytemuck::cast_slice(&[
                    Vertex { position: [0.2, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
                    Vertex { position: [0.8, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
                    Vertex { position: [0.5, 0.5, 0.0], color: [0.0, 0.0, 1.0] },
                ]),
                &[0, 1, 2],
                &context.device,
            ),
            uniforms: Vec::new(),
            pipeline: pid,
        };
        renderer.register_object(obj1);

        let obj2 = render::Renderable {
            mesh: render::Mesh::new(
                bytemuck::cast_slice(&[
                    Vertex { position: [-0.8, -0.5, 0.0], color: [1.0, 0.0, 0.0] },
                    Vertex { position: [-0.2, -0.5, 0.0], color: [1.0, 0.0, 0.0] },
                    Vertex { position: [-0.5, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
                ]),
                &[0, 1, 2],
                &context.device,
            ),
            uniforms: Vec::new(),
            pipeline: pid,
        };
        renderer.register_object(obj2);

        Self {}
    }

    fn frame(&mut self, _: &mut crate::render::RenderContext, renderer: &mut crate::render::Renderer) {
        renderer.commands.push(render::RenderCommand { object_id: 0 });
        renderer.commands.push(render::RenderCommand { object_id: 1 });
    }
}
